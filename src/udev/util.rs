extern crate libc;

use std::raw::Slice;
use std::intrinsics;
use libc::{ENOMEM, c_int};
use std::mem;
use std::ptr;
use std;

pub unsafe fn c_to_str<'a>(s: *const libc::c_char) -> Option<&'a str> {
    if s.is_null() {
        None
    } else {
        let mut cur = s;
        let mut len = 0;
        while *cur != 0 {
            len += 1;
            cur = cur.offset(1);
        }
        let slice = mem::transmute(Slice { data: s, len: len });
        std::str::from_utf8(slice)
    }
}

#[inline(always)]
pub fn oom() -> ! {
    unsafe { intrinsics::abort(); }
}

pub fn handle_error(err: i32) {
    match err {
        0 => (),
        x if x == -ENOMEM => oom(),
        _ => panic!("Unhandled udev error.")
    }
}

#[cfg(any(target_os = "linux", target_os = "android"))]
fn errno_location() -> *mut c_int {
    extern {
        fn __errno_location() -> *mut c_int;
    }
    unsafe {
        __errno_location()
    }
}

pub fn get_errno() -> c_int {
    unsafe {
        *errno_location()
    }
}

pub fn set_errno(value: c_int) {
    unsafe {
        *errno_location() = value;
    }
}

pub fn check_errno<I, T: ptr::RawPtr<I>>(f: || -> T) -> Result<Option<T>, c_int> {
    set_errno(0);
    let result = f();
    if result.is_null() {
        match get_errno() {
            ENOMEM => oom(),
            0 => Ok(None),
            e => Err(e)
        }
    } else {
        Ok(Some(result))
    }
}

