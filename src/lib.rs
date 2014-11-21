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

#![feature(unsafe_destructor, if_let, globs)]

extern crate alloc;
extern crate libc;

pub use udev::udev::Udev;

mod udev;

pub mod hwdb {
    pub use udev::hwdb::{
        Hwdb,
        Query,

        HwdbIterator,
    };
}
pub mod device {
    pub use udev::device::{
        Device,
        Devnum,
        Type,

        TagIterator,
        AttributeIterator,
        DevlinkIterator,
        PropertyIterator,
    };
}
pub mod enumerator {
    pub use udev::enumerator::{
        Enumerator,

        DeviceIterator,
        DevicePathIterator,
    };
}
pub mod monitor {
    pub use udev::monitor::{
        Monitor,
        Event,
        Action,

        MonitorIterator,
    };
}

#[cfg(test)]
mod test {
    use Udev;

    #[test]
    fn test_ttys() {
        let udev = Udev::new();
        let mut vec = Vec::with_capacity(64);
        let mut found_tty = false;
        for dev in udev.enumerator().match_subsystem("tty").scan_devices().iter() {
            assert!(dev.subsystem().unwrap() == "tty");
            if dev.sysname().starts_with("tty") {
                match dev.sysnum() {
                    Some(num) => vec.push(num),
                    None => {
                        assert!(!found_tty);
                        found_tty = true;
                    }
                }
            }
        }

        vec.sort();
        assert!(vec.into_iter().zip(range(0u64, 64u64)).all(|(i, j)| i == j));
    }
}
