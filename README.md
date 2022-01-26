# mpk-protector

This crate provides a compiler plugin that can automatically provide wrapper functions for all foreign functions in the crate being annotated. 

It works in concert with pkmallocator to automate and manage partitioning heap data between `trusted` and `untrusted` regions of the heap. By default all Rust allocations are trusted, and any allocation coming from C is untrusted. Trusted pages have MPK keys associated with them, and this plugin wraps foreign APIs to disable access to these pages when leaving Rust. As a consequence any data leaving Rust must come from the untrusted region, so when passing a pointer or other data structures to C, it is important to either ensure that the object is originally allocated from the untrusted region, or to make a copy whose backing memory is from the untrusted region. Which method is appropriate will be determined by the developer, and the needs of their project.

To facilitate this pkmallocator provides a useful macro to help manage makeing untrusted allocations.

## Usage

``` rust
#![feature(plugin, custom_attribute)] // declares intention to use plugin and annotation
#![plugin(mpk_protector)] // imports plugin
#![mpk_protector] // annotates the crate/module (can be directly applied to desired foreign module)

use::os::raw::c_char;

extern "C" {
  pub fn use_ptr(ptr: *const c_char) -> *mut c_char;
  static my_buff: *mut c_char;
}
```

This will move the extern declaration into a hidden module, and create a wrapper function with the same name. In the example above, that would create a new function with the following behavior(code below is only an example implementation):

```rust
pub fn use_ptr(ptr: *const c_char) -> *mut c_char{
  // manipulate mpk registers
  let p = read_mpk();
  set_pkru(no_access);
  
  // make original function call
  let r =some::hidden::module::name::use_ptr(ptr)
  
  // restore previous mpk state
  set_pkru(p);
  
  // return result
  return r;
}
```

The newly imported pkmallocator crate provides a global allocator, and a macro `untrusted!{}` that can be used to wrap code blocks, and causes any allocation within the block to come from the untrusted region

Additionally the Cargo.toml should have the following additions made to its dependencies

```toml
[dependencies]
mpk_protector = { git = "https://github.com/securesystemslab/pkru-safe-mpk-protector.git" }
pkmallocator = { git = "https://github.com/securesystemslab/pkru-safe-pkmallocator.git" }
mpk = { git = "https://github.com/securesystemslab/pkru-safe-mpk-libc.git" }
```

