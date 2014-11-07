#![feature(unsafe_destructor, if_let)]
extern crate libc;

pub use udev::udev::Udev;

mod udev;

pub mod hwdb {
    pub use udev::hwdb::{
        Hwdb,
        HwdbQuery,
        HwdbIterator,
    };
}
pub mod device {
    pub use udev::device::{
        Device,
        Devnum,
        DeviceType,
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
        MonitorIterator,
        Event,

        // My kingdome for a scope
        Action,
        AddAction,
        RemoveAction,
        ChangeAction,
        MoveAction,
        OfflineAction,
        OnlineAction,
        OtherAction,
    };
}

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

