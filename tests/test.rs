#![feature(libc)]
#[macro_use]
extern crate pkmallocator;
extern crate libc;
extern crate pkalloc;
extern crate untrusted;

use std::ffi::CString;
use std::sync::Arc;
use untrusted::{change_vector, get_last_val, use_arc, use_ptr};

struct MyType {
    val: f64,
}

fn mkbox(val: i32) -> Box<i32> {
    Box::new(val)
}

#[test]
fn use_rust_ptr() {
    let msg;

    untrusted!({
        msg = CString::new("Hello World!").unwrap();
    });
    let mut rust_msg = CString::new("Have new Stuff!").unwrap();
    println!("{:?}", rust_msg);
    let cmsg = msg.as_ptr();
    println!("string address \t: {:?}", &cmsg);
    unsafe {
        let ptr = use_ptr(cmsg);
        assert_ne!(cmsg, ptr);
        assert_eq!(*cmsg, *ptr);
        libc::free(ptr as *mut libc::c_void);
    }

    let old_num = 42;
    let uold = &old_num as *const i32;

    let mut num_box: Box<i32> = untrusted!({ mkbox(0) });
    *num_box = unsafe { get_last_val(uold) };
    assert_ne!(old_num, *num_box);
    *num_box = 13;
    let num_ptr = Box::into_raw(num_box);
    println!("Box address \t: {:p}", num_ptr);
    let uold = num_ptr as *const i32;
    num_box = unsafe { Box::from_raw(num_ptr) };
    *num_box = unsafe { get_last_val(uold) };
    assert_eq!(old_num, *num_box);

    rust_msg = CString::new("Done!").unwrap();
    println!("{:?}", rust_msg);
}

fn make_vec() -> Vec<i32> {
    //untrusted!({
    vec![1, 2, 3]
    //})
}

fn make_safe_vec() -> Vec<MyType> {
    vec![
        MyType { val: 1.0 },
        MyType { val: 2.0 },
        MyType { val: 3.0 },
        MyType { val: 4.0 },
        MyType { val: 5.0 },
        MyType { val: 6.0 },
        MyType { val: 7.0 },
        MyType { val: 8.0 },
        MyType { val: 9.0 },
        MyType { val: 10.0 },
    ]
}

#[test]
fn use_rust_vector() {
    let mut num_box = 0;
    println!("Addres of stack value: {:p}", &num_box as *const _);
    let mut v = untrusted!({ make_vec() });
    // let mut v = make_vec() ;
    println!("Addres of vec: {:p}", &v as *const _);

    for i in 0..v.len() {
        print!("{:?},", v[i]);
    }
    print!("\n");

    {
        let buf = &v;
        let old_num = buf[0].clone();
        //let new_num = buf[1].clone();
        let buf_ptr = buf.as_ptr();
        //let uold = pkmallocator::untrusted_ty {
        //val: buf_ptr as *const i32,
        //};
        let uold = buf_ptr as *const i32;

        num_box = unsafe { get_last_val(uold) };
        assert_ne!(old_num, num_box);
        //let uold = pkmallocator::untrusted_ty {
        //val: buf_ptr as *const i32,
        //};
        let uold = buf_ptr as *const i32;
        num_box = unsafe { get_last_val(uold) };
        assert_eq!(old_num, num_box);
        assert!(v.len() < 50);
        println!("{:?}", buf_ptr);
    }

    let mut len = v.len() as i32;
    if num_box == 1 {
        len = 25 as i32;
        for i in v.len() as i32..len {
            v.push(i + 1);
        }
    }

    let buf = &mut v;
    let old_num = buf[0].clone();
    //let new_num = buf[1].clone();
    let buf_ptr = buf.as_mut_ptr();
    //let uold = pkmallocator::untrusted_ty {
    //val: unsafe { buf_ptr.offset(len as isize - 1) } as *const i32,
    //};
    let uold = unsafe { buf_ptr.offset(len as isize - 1) } as *const i32;

    num_box = unsafe { get_last_val(uold) };
    assert_eq!(old_num, num_box);
    unsafe {
        let success = change_vector(buf_ptr.offset(0), buf_ptr.offset(1));
        assert!(success == 1);
    }

    for i in 0..buf.len() {
        println!("{:?} - {:p}", buf[i], unsafe { buf_ptr.offset(i as isize) });
    }
    println!("{:?}", uold);
}

#[test]
fn use_arc_test() {
    let arc = untrusted!({ Arc::new([1, 2, 3]) });
    //let arc = Arc::new([1, 2, 3]);
    let sum: i32 = arc.iter().sum();

    let (arc, ffi_sum) = use_arc(arc);
    assert_eq!(sum, ffi_sum);
    let new_sum = arc.iter().sum();
    assert_eq!(sum, new_sum);

    let mut v = make_safe_vec();
    unsafe {
        let offset = v.len() as isize - 1;
        let buf = &mut v;
        let buf_ptr = buf.as_mut_ptr();
        println!("buf_ptr address = {:p}", buf_ptr.offset(offset));
        assert!(pkalloc::pk_is_safe_addr(buf_ptr as *mut u8));
    }
    assert_eq!(
        v.iter().fold(0.0, |sum: f64, mt: &MyType| sum + mt.val),
        55.0
    )
}
