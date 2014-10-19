use std::kinds::marker::NoSync;
use std::io::IoError;

use device;
use util;
use hwdb;
use monitor;
use enumerator;
use libudev_c;

use device::{Device, DeviceType, Devnum};
use hwdb::{Hwdb};
use monitor::Monitor;
use enumerator::{DeviceEnumerator, SubsystemEnumerator};

pub struct Udev {
    // Not thread safe. As all children will hold a reference, this makes everything safe.
    nosync: NoSync,
    udev: libudev_c::udev
}

// Crate private
pub unsafe fn get_udev_ctx(udev: &Udev) -> libudev_c::udev {
    udev.udev
}

impl Udev {
    pub fn new() -> Udev {
        let udev = unsafe { libudev_c::udev_new() };
        // I don't care about errno. NULL == oom.
        if udev.is_null() {
            util::oom();
        }
        Udev { nosync: NoSync, udev: udev }
    }

    fn create_monitor(&self, name: &str) -> Result<Monitor, IoError>  {
        name.with_c_str(|name| monitor::monitor(self, || unsafe {
            libudev_c::udev_monitor_new_from_netlink(self.udev, name)
        }))
    }

    pub fn monitor(&self) -> Result<Monitor, IoError> {
        self.create_monitor("udev")
    }

    pub fn monitor_kernel(&self) -> Result<Monitor, IoError> {
        self.create_monitor("kernel")
    }

    pub fn hwdb(&self) -> Result<Hwdb, i32> {
        hwdb::hwdb(self)
    }

    pub fn device(&self, path: &Path) -> Option<Device> {
        match path.with_c_str(|path| util::check_errno(|| unsafe {
            libudev_c::udev_device_new_from_syspath(self.udev, path)
        })) {
            Ok(Some(dev)) => Some(unsafe { device::device(self, dev) }),
            _ => None
        }
    }

    pub fn device_from_devnum(&self, ty: DeviceType, devnum: Devnum) -> Option<Device> {
        match util::check_errno(|| unsafe {
            libudev_c::udev_device_new_from_devnum(self.udev, ty.to_char(), devnum)
        }) {
            Ok(Some(dev)) => Some(unsafe { device::device(self, dev) }),
            _ => None
        }
    }
    pub fn device_from_subsystem_sysname(&self, subsystem: &str, sysname: &str) -> Option<Device> {
        match subsystem.with_c_str(|subsystem| sysname.with_c_str(|sysname| util::check_errno(|| unsafe {
            libudev_c::udev_device_new_from_subsystem_sysname(self.udev, subsystem, sysname)
        }))) {
            Ok(Some(dev)) => Some(unsafe { device::device(self, dev) }),
            _ => None
        }
    }

    pub fn devices(&self) -> DeviceEnumerator {
        enumerator::device_enumerator(self, None)
    }

    pub fn subsystems(&self) -> SubsystemEnumerator {
        enumerator::subsystem_enumerator(self)
    }
}

impl Drop for Udev {
    fn drop(&mut self) {
        unsafe { libudev_c::udev_unref(self.udev) };
    }
}
