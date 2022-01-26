// MIT License
// mpk-protector/src/lib.rs - mpk-protector
//
// Copyright 2018 Paul Kirth
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE

#![feature(plugin_registrar, rustc_private, custom_attribute)]

/// Design
///
/// This plugin protects Rust internal data by using Intel MPK. Untrusted functions
/// will be wrapped w/ an annotation. Each annotated function will disable the access
/// through PKRU, complete the call, and then re-enable access through PKRU. Furthermore,
/// each buffer passed out through the protected API must have been allocated from the
/// unsafe region. Any safe pointers which leak will cause a segmentation fault when these
/// locations are accessed, provided PKRU is set correctly.
///
/// we walk the AST, and at any point when we find an annotated function we will wrap all
/// calls to that function to prevent PKRU from leaking.
/// Later we will try to trace the origin of its data to the unsafe region.
/// This may be achievable by extending to borrow checker, or enhancing the AST w/ Allocator state
///
extern crate itertools;
extern crate mpk;
extern crate proc_macro;
extern crate rustc;
extern crate rustc_plugin;
extern crate syntax;
extern crate syntax_pos;

use itertools::Itertools;
use rustc_plugin::Registry;
use syntax::ast;
use syntax::ast::{ForeignItem, ForeignItemKind, VisibilityKind};
use syntax::ast::{FunctionRetTy, Item, ItemKind, Mac, MetaItem, Mod};
use syntax::errors;
use syntax::ext::base::{Annotatable, ExtCtxt, SyntaxExtension, SyntaxExtensionKind};
use syntax::mut_visit::{self, ExpectOne, MutVisitor};
use syntax::parse::parser::Parser;
use syntax::parse::{new_parser_from_source_str, PResult, ParseSess};
use syntax::print::pprust;
use syntax::ptr::P;
use syntax::source_map::{FilePathMapping, Span};
use syntax_pos::symbol::Symbol;

use std::path::PathBuf;

macro_rules! panictry {
    ($e:expr) => {{
        use errors::FatalError;
        use std::result::Result::{Err, Ok};
        match $e {
            Ok(e) => e,
            Err(mut e) => {
                e.emit();
                FatalError.raise()
            }
        }
    }};
}

fn with_error_checking_parse<'a, T, F>(s: String, ps: &'a ParseSess, f: F) -> T
where
    F: FnOnce(&mut Parser<'a>) -> PResult<'a, T>,
{
    let mut p = string_to_parser(&ps, s);
    let x = panictry!(f(&mut p));
    p.sess.span_diagnostic.abort_if_errors();
    x
}

/// Map string to parser (via tts)
pub fn string_to_parser<'a>(ps: &'a ParseSess, source_str: String) -> Parser<'a> {
    new_parser_from_source_str(ps, PathBuf::from("bogofile").into(), source_str)
}

/// Parse a string, return an item
pub fn string_to_item(source_str: String) -> Option<P<ast::Item>> {
    let ps = ParseSess::new(FilePathMapping::empty());
    with_error_checking_parse(source_str, &ps, |p| p.parse_item())
}

struct Untrusted {}

impl MutVisitor for Untrusted {
    fn visit_mod(&mut self, mut m: &mut Mod) {
        // finish processing the module as normal
        mut_visit::noop_visit_mod(m, self);

        let mut new_items: Vec<P<Item>> = Vec::new();
        let mut new_foreign_items: Vec<P<Item>> = Vec::new();
        let header = vec![
            "extern crate pkmallocator;\n",
            "extern crate mpk;\n",
            "use mpk::pkey_set_panic;\n",
            "use pkmallocator::PkAlloc;\n",
            "use pkmallocator::__untrusted_gate_exit;\n",
            "use pkmallocator::__untrusted_gate_enter;\n",
            "use pkmallocator::untrusted_ty;\n",
        ];
        for it in header {
            new_items.push(string_to_item(it.to_string()).unwrap());
        }

        // TODO: consider generating the module name
        // setup skeleton for hidden module
        let mod_str = "mod secret_mpk_funcs {\nuse super::*;\n\n}\n";
        let new_mod_item = string_to_item(mod_str.to_string()).unwrap();

        let mut setup_module = false; // is a new module required

        // modify any foreign modules that exist within this module
        m.items = m
            .items
            .iter()
            .cloned()
            .filter_map(|item_ptr| {
                let mut is_none = false;
                let item_ret = item_ptr.map(|mut item| {
                    // get mutable copies of the target item
                    // Required to avoid problems with borrowck
                    let mut clean_item = item.clone(); // a replacement copy of the item

                    // only look at the foreign mod
                    if let ItemKind::ForeignMod(ref mut fm) = item.node {
                        self.process_foreign_module(
                            &mut new_items,
                            &mut new_foreign_items,
                            &mut setup_module,
                            &mut is_none,
                            &mut clean_item,
                            fm,
                        )
                    }
                    clean_item
                });

                // Avoid keeping empty foreign modules around
                if is_none {
                    None
                } else {
                    Some(item_ret)
                }
            })
            .collect();

        if setup_module {
            let v = new_mod_item.map(|mut item| {
                if let ItemKind::Mod(ref mut new_mod) = item.node {
                    new_mod.items.append(&mut new_foreign_items);
                }
                item.clone()
            });
            new_items.push(v);
        }

        // insert all new Items into this module
        m.items.append(&mut new_items);
    }

    fn visit_mac(&mut self, mac: &mut Mac) {
        mut_visit::noop_visit_mac(mac, self)
    }
}

impl Untrusted {
    fn make_wrapper(&mut self, func: &ForeignItem) -> P<Item> {
        let old_name = func.ident.to_string();
        let new_name = "secret_mpk_funcs::".to_string() + &old_name;
        let vis = match func.vis.node {
            VisibilityKind::Public => "pub",
            _ => "",
        };
        let mut args_str = String::new();
        let mut ident_str = String::new();
        let mut ret_str = String::new();
        let mut idents = vec![];
        let mut types = vec![];
        let mut string_vec = vec![];

        if let ForeignItemKind::Fn(ref decl, ref _generics) = func.node {
            for arg in &decl.inputs {
                let local_id = pprust::pat_to_string(&arg.pat);
                let local_ty = pprust::ty_to_string(&arg.ty);
                idents.push(local_id.clone());
                types.push(local_ty.clone());

                //string_vec.push(format!{"{}: untrusted_ty<{}>",  local_id, local_ty});
                string_vec.push(format! {"{}: {}", local_id, local_ty});
            }
            args_str = format! {"{}", string_vec.iter().join(", ")};
            if idents.is_empty() {
                ident_str = format! {"{}", idents.iter().join(", ")};
            } else {
                //ident_str = format!{"{}.val", idents.iter().join(", ")};
                ident_str = format! {"{}", idents.iter().join(", ")};
            }
            ret_str = match &decl.output {
                FunctionRetTy::Default(_) => "()".to_owned(),
                FunctionRetTy::Ty(x) => pprust::ty_to_string(x),
            };
        }

        let mpk_off = "__untrusted_gate_exit()";
        let mpk_on = "__untrusted_gate_enter()";

        let fn_str = format!(
            "#[allow(non_snake_case)]\n\
             #[untrusted] \
             #[inline(never)] \
             {} unsafe extern \"C\" \
             fn {}({}) -> {} {{\
             use pkmallocator::{{__untrusted_gate_enter, __untrusted_gate_exit }};\n\
             {};\n\
             let transparent_return_value = {}({});\n\
             {};\n\
             transparent_return_value \
             }}",
            vis, old_name, args_str, ret_str, mpk_off, new_name, ident_str, mpk_on
        );
        string_to_item(fn_str).unwrap()
    }

    fn create_mod_wrappers(
        &mut self,
        new_items: &mut Vec<P<Item>>,
        setup_module: &mut bool,
        fm: &ast::ForeignMod,
    ) {
        //loop through the foreign items
        for f_item in fm.items.iter() {
            // generate a function wrapper and stick it in the module
            if let ForeignItemKind::Fn(ref _decl, ref _generics) = f_item.node {
                *setup_module = true;
                let func_wrapper = self.make_wrapper(&f_item);
                new_items.push(func_wrapper);
            }
        }
    }

    fn update_visibility(fake_mod: &mut ast::ForeignMod, f_node: Option<P<Item>>) {
        for mut i in fake_mod.items.iter_mut() {
            let mut tmp_span = i.vis.clone();
            tmp_span.node = VisibilityKind::Public;
            i.vis = tmp_span;
            if let Some(ref v) = f_node {
                let mut attrs = i.attrs.clone();
                for a in &v.attrs {
                    attrs.push(a.clone());
                }
                //i.attrs = attrs.into_iter().unique();
                i.attrs = attrs;
            };
        }
    }

    fn process_foreign_module(
        &mut self,
        mut new_items: &mut Vec<P<Item>>,
        new_foreign_items: &mut Vec<P<Item>>,
        mut setup_module: &mut bool,
        is_none: &mut bool,
        mut clean_item: &mut Item,
        mut fm: &mut ast::ForeignMod,
    ) -> () {
        let mut fake_item = clean_item.clone(); // a new copy to insert into a new module
        self.create_mod_wrappers(&mut new_items, &mut setup_module, &fm);
        // get mutable copy of the foreign module, and its items
        let mut fake_mod = fm.clone();
        let fake_item_list = fm.items.clone();
        // sort items into functions/not functions
        let (function_item_list, other_item_list): (Vec<ForeignItem>, Vec<ForeignItem>) =
            fake_item_list.into_iter().partition(|ref x| match x.node {
                ForeignItemKind::Fn(..) => true,
                _ => false,
            });
        let f_node = new_items.clone().into_iter().find(|ref x| match x.node {
            ItemKind::Fn(..) => true,
            _ => false,
        });
        //let optA = f_node.unwrap().attrs.into_iter().find(|y| y == OtherUntrusted);
        //replace non mutable lists w/ new copies
        fake_mod.items = function_item_list;
        fm.items = other_item_list;
        Untrusted::update_visibility(&mut fake_mod, f_node);
        // make new module
        fake_item.node = ItemKind::ForeignMod(fake_mod.clone());
        new_foreign_items.push(P(fake_item));
        // borrow and mutate clean item
        let mut temp = &mut clean_item;
        temp.node = ItemKind::ForeignMod(fm.clone());
        // determine if the original foreign mod had
        // items other than functions
        if fm.items.len() == 0 {
            *is_none = true; // set external boolean
        }
    }
}

fn mpk_create(_ecx: &mut ExtCtxt, _sp: Span, _meta: &MetaItem, a: Annotatable) -> Annotatable {
    match a {
        Annotatable::Item(i) => Annotatable::Item(
            Untrusted {}
                .flat_map_item(i)
                .expect_one("expected exactly one item"),
        ),
        a => a,
    }
}

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    let name = Symbol::intern("mpk_protector");
    let func = SyntaxExtension::default(
        SyntaxExtensionKind::LegacyAttr(Box::new(mpk_create)),
        reg.sess.edition(),
    );
    reg.register_syntax_extension(name, func);
}
