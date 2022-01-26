// untrusted/src/lib.rs - PKRU-Safe
//
// Copyright 2018 Paul Kirth
//
// Permission is hereby granted, free of charge, to any person obtaining a
// copy of this software and associated documentation files (the "Software"),
// to deal in the Software without restriction, including without limitation
// the rights to use, copy, modify, merge, publish, distribute, sublicense,
// and/or sell copies of the Software, and to permit persons to whom the
// Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
// FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS
// IN THE SOFTWARE.

#![feature(plugin, custom_attribute)]
#![feature(macros_in_extern)]
#![feature(libc)]
#![plugin(mpk_protector)]
#![mpk_protector]
#![allow(unused)]

use std::os::raw::{c_char, c_int, c_uint};
use std::sync::Arc;

extern "C" {
    pub fn use_ptr(ptr: *const c_char) -> *mut c_char;
    pub fn use_two_ptr(charptr: *const c_char, intptr: *mut i32) -> *mut c_char;
    pub fn get_last_val(intptr: *const i32) -> i32;
    pub fn change_vector(first: *mut i32, second: *mut i32) -> i32;
    pub fn use_arc_array(array: *const i32, size: c_uint) -> i32;
    pub fn access_vec(array: *const i32, size: c_uint) -> i32;
    static my_buff: *mut c_char;
}

pub fn use_arc(arc_array: Arc<[i32]>) -> (Arc<[i32]>, i32) {
    unsafe {
        let ptr = Arc::into_raw(arc_array);
        let size = (*ptr).len() as c_uint;
        let array_ptr = (*ptr).as_ptr();
        let sum = use_arc_array(array_ptr, size);

        (Arc::from_raw(ptr), sum)
    }
}
