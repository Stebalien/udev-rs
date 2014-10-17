use std::kinds::marker::NoSync;

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

pub unsafe fn new_enumerator<'u>(udev: &'u Udev) -> libudev_c::udev_enumerate {
    libudev_c::udev_enumerate_new(udev.udev)
}

impl Udev {
    pub fn new() -> Udev {
        let udev = unsafe { libudev_c::udev_new() };
        if udev.is_null() {
            util::oom();
        }
        Udev { nosync: NoSync, udev: udev }
    }

    fn create_monitor(&self, name: &str) -> Monitor {
        let monitor = name.with_c_str(|name| unsafe {
            libudev_c::udev_monitor_new_from_netlink(self.udev, name)
        });
        // TODO: check errno for oom etc...
        // TODO Return result?
        assert!(!monitor.is_null(), "failed to create the requested monitor.");

        unsafe {
            monitor::monitor(self, monitor)
        }
    }

    pub fn monitor(&self) -> Monitor {
        self.create_monitor("udev")
    }

    pub fn monitor_kernel(&self) -> Monitor {
        self.create_monitor("kernel")
    }

    pub fn hwdb(&self) -> Result<Hwdb, ()> {
        hwdb::hwdb(self, unsafe { libudev_c::udev_hwdb_new(self.udev) })
    }

    pub fn device(&self, path: &Path) -> Option<Device> {
        path.with_c_str(|path| unsafe {
            device::device(
                self,
                libudev_c::udev_device_new_from_syspath(self.udev, path),
                false)
        })
    }
    pub fn device_from_devnum(&self, ty: DeviceType, devnum: Devnum) -> Option<Device> {
        unsafe {
            device::device(
                self,
                libudev_c::udev_device_new_from_devnum(self.udev, ty.to_char(), devnum),
                false)
        }
    }
    pub fn device_from_subsystem_sysname(&self, subsystem: &str, sysname: &str) -> Option<Device> {
        subsystem.with_c_str(|subsystem| sysname.with_c_str(|sysname| unsafe {
            device::device(
                self,
                libudev_c::udev_device_new_from_subsystem_sysname(self.udev, subsystem, sysname),
                false)
        }))
    }

    pub fn devices(&self) -> DeviceEnumerator {
        unsafe {
            enumerator::device_enumerator(self, None)
        }
    }

    pub fn subsystems(&self) -> SubsystemEnumerator {
        unsafe {
            enumerator::subsystem_enumerator(self)
        }
    }
}

impl Drop for Udev {
    fn drop(&mut self) {
        unsafe { libudev_c::udev_unref(self.udev) };
    }
}
