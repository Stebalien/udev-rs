// This file is part of udev-rs.
// 
// Copyright 2014 Steven Allen <steven@stebalien.com>
// 
// udev-rs is free software; you can redistribute it and/or modify it
// under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation; either version 2.1 of the License, or
// (at your option) any later version.
// 
// udev-rs is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// Lesser General Public License for more details.
// 
// You should have received a copy of the GNU Lesser General Public License
// along with udev-rs; If not, see <http://www.gnu.org/licenses/>.

use std::raw::Slice;
use std::{mem, ptr, str};

use libc::{ENOMEM, c_int, c_char};
use alloc::oom;

pub unsafe fn c_to_str<'a>(s: *const c_char) -> Option<&'a str> {
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
        str::from_utf8(slice)
    }
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

