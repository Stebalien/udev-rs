extern crate libc;

use std::raw::Slice;
use std::intrinsics;
use libc::ENOMEM;
use std::mem;
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
        x if x == -ENOMEM => unsafe { intrinsics::abort() },
        _ => fail!("Unhandled udev error.")
    }
}

