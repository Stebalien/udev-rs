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

use std::kinds::marker::NoSync;
use std::io::IoError;

use libc::{
    fcntl,
    O_NONBLOCK,
    F_SETFL,
    F_GETFL,
    ENOMEM,
    EINVAL,
};
use alloc::oom;

use udev::{
    device,
    util,
    hwdb,
    monitor,
    enumerator,
    libudev_c,
};
use udev::device::{
    Device,
};
use udev::hwdb::Hwdb;
use udev::monitor::Monitor;
use udev::enumerator::Enumerator;

pub struct Udev {
    // Not thread safe. As all children will hold a reference, this makes everything safe.
    nosync: NoSync,
    udev: libudev_c::udev
}

impl Udev {
    /// Create a new udev handle.
    pub fn new() -> Udev {
        let udev = unsafe { libudev_c::udev_new() };
        // I don't care about errno. NULL == oom.
        if udev.is_null() {
            oom();
        }
        Udev { nosync: NoSync, udev: udev }
    }

    fn create_monitor(&self, name: &str) -> Result<Monitor, IoError>  {
        let monitor = match name.with_c_str(|name| util::check_errno(|| unsafe {
            libudev_c::udev_monitor_new_from_netlink(self.udev, name)
        })) {
            Ok(Some(monitor))       => monitor,
            Err(EINVAL) | Ok(None)  => panic!("BUG"),
            Err(e)                  => return Err(IoError::from_errno(e as uint, true))
        };
        let fd = unsafe {
            libudev_c::udev_monitor_get_fd(monitor)
        };

        let old_val = unsafe { fcntl(fd, F_GETFL) };
        if old_val == -1 || unsafe { fcntl(fd, F_SETFL, old_val & !O_NONBLOCK) == -1 } {
            return match util::get_errno() {
                ENOMEM | EINVAL => panic!("BUG"),
                e => Err(IoError::from_errno(e as uint, true))
            }
        }

        Ok(unsafe { monitor::monitor(self, monitor) })
    }

    /// Monitor udev events.
    ///
    /// # Error
    ///
    /// This will return an error if you're running in an environment without access to netlink.
    pub fn monitor(&self) -> Result<Monitor, IoError> {
        self.create_monitor("udev")
    }

    /// Monitor kernel events.
    ///
    /// # Error
    ///
    /// This method will return an error if you're running in an environment without access to
    /// netlink.
    ///
    /// # Safety Notes
    ///
    /// This method is marked unsafe due to the following warning found in libudev:
    ///
    /// > Applications should usually not connect directly to the
    /// > "kernel" events, because the devices might not be useable
    /// > at that time, before udev has configured them, and created
    /// > device nodes. Accessing devices at the same time as udev,
    /// > might result in unpredictable behavior. The "udev" events
    /// > are sent out after udev has finished its event processing,
    /// > all rules have been processed, and needed device nodes are
    /// > created.
    pub unsafe fn monitor_kernel(&self) -> Result<Monitor, IoError> {
        self.create_monitor("kernel")
    }

    /// Create a new hardware database handle.
    ///
    /// # Error
    ///
    /// On error, this method will return either Err(errno) or Err(0). Err(errno) indicates a
    /// problem reading the hardware database and Err(0) indicates that the hardware database is
    /// corrupt.
    pub fn hwdb(&self) -> Result<Hwdb, i32> {
        match util::check_errno(|| unsafe {
            libudev_c::udev_hwdb_new(self.udev)
        }) {
            Ok(Some(hwdb))  => Ok(unsafe { hwdb::hwdb(self, hwdb) }),
            Ok(None)        => Err(0i32),
            Err(EINVAL)     => panic!("BUG"),
            Err(e)          => Err(e)
        }
    }

    /// Lookup a device by sys path.
    pub fn device(&self, path: &Path) -> Option<Device> {
        match path.with_c_str(|path| util::check_errno(|| unsafe {
            libudev_c::udev_device_new_from_syspath(self.udev, path)
        })) {
            Ok(Some(dev)) => Some(unsafe { device::device(self, dev) }),
            _ => None
        }
    }

    /// Lookup a device by device type and device number.
    pub fn device_from_devnum(&self, ty: device::Type, devnum: device::Devnum) -> Option<Device> {
        match util::check_errno(|| unsafe {
            libudev_c::udev_device_new_from_devnum(self.udev, ty.to_char(), devnum)
        }) {
            Ok(Some(dev)) => Some(unsafe { device::device(self, dev) }),
            _ => None
        }
    }

    /// Lookup a device by subsystem and sysname
    pub fn device_from_subsystem_sysname(&self, subsystem: &str, sysname: &str) -> Option<Device> {
        match subsystem.with_c_str(|subsystem| sysname.with_c_str(|sysname| util::check_errno(|| unsafe {
            libudev_c::udev_device_new_from_subsystem_sysname(self.udev, subsystem, sysname)
        }))) {
            Ok(Some(dev)) => Some(unsafe { device::device(self, dev) }),
            _ => None
        }
    }

    /// Create a device enumerator.
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
