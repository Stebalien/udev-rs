use std::kinds::marker::NoSync;
use std::io::IoError;
use libc::{fcntl, O_NONBLOCK, F_SETFL, F_GETFL, ENOMEM, EINVAL};

use device;
use util;
use hwdb;
use monitor;
use enumerator;
use libudev_c;

use device::{Device, DeviceType, Devnum};
use hwdb::{Hwdb};
use monitor::Monitor;
use enumerator::Enumerator;

pub struct Udev {
    // Not thread safe. As all children will hold a reference, this makes everything safe.
    nosync: NoSync,
    udev: libudev_c::udev
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
        let monitor = match name.with_c_str(|name| util::check_errno(|| unsafe {
            libudev_c::udev_monitor_new_from_netlink(self.udev, name)
        })) {
            Ok(Some(monitor))       => monitor,
            Err(EINVAL) | Ok(None)  => fail!("BUG"),
            Err(e)                  => return Err(IoError::from_errno(e as uint, true))
        };
        let fd = unsafe {
            libudev_c::udev_monitor_get_fd(monitor)
        };

        let old_val = unsafe { fcntl(fd, F_GETFL) };
        if old_val == -1 || unsafe { fcntl(fd, F_SETFL, old_val & !O_NONBLOCK) == -1 } {
            return match util::get_errno() {
                ENOMEM | EINVAL => fail!("BUG"),
                e => Err(IoError::from_errno(e as uint, true))
            }
        }

        Ok(unsafe { monitor::monitor(self, monitor) })
    }

    pub fn monitor(&self) -> Result<Monitor, IoError> {
        self.create_monitor("udev")
    }

    pub unsafe fn monitor_kernel(&self) -> Result<Monitor, IoError> {
        self.create_monitor("kernel")
    }

    pub fn hwdb(&self) -> Result<Hwdb, i32> {
        match util::check_errno(|| unsafe {
            libudev_c::udev_hwdb_new(self.udev)
        }) {
            Ok(Some(hwdb))  => Ok(unsafe { hwdb::hwdb(self, hwdb) }),
            Ok(None)        => Err(0i32),
            Err(EINVAL)     => fail!("BUG"),
            Err(e)          => Err(e)
        }
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

    pub fn enumerator(&self) -> Enumerator {
        unsafe {
            enumerator::enumerator(
                self, util::check_errno(|| {
                    libudev_c::udev_enumerate_new(self.udev)
                }).unwrap().unwrap())
        }
    }
}

impl Drop for Udev {
    fn drop(&mut self) {
        unsafe { libudev_c::udev_unref(self.udev) };
    }
}
