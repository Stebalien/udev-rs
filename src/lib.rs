#![feature(unsafe_destructor, if_let)]
extern crate libc;

pub use udev::Udev;
pub use hwdb::{
    Hwdb,
    HwdbQuery
};
pub use device::{
    Device,
    Devnum,
    DeviceType,
};
pub use enumerator::Enumerator;
pub use monitor::Monitor;

mod libudev_c;
mod udev;
mod hwdb;
mod util;
mod device;
mod enumerator;
mod monitor;
mod iterator;

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

