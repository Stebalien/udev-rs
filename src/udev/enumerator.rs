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

use std::ptr;

use udev::{
    libudev_c,
    util,
    iterator,
    device,
};
use udev::udev::Udev;
use udev::device::Device;
use udev::iterator::{
    MappedIterator,
    FilterMappedIterator,
};

pub struct Enumerator<'u> {
    udev: &'u Udev,
    enumerator: libudev_c::udev_enumerate
}

// Crate Private
pub unsafe fn enumerator<'u>(udev: &'u Udev, enumerator: libudev_c::udev_enumerate) -> Enumerator<'u> {
    Enumerator {
        udev: udev,
        enumerator: enumerator
    }
}

#[doc(hidden)]
pub type DeviceIterator<'e, 'u: 'e> = FilterMappedIterator<'e, Enumerator<'u>, Device<'u>>;
#[doc(hidden)]
pub type DevicePathIterator<'p> = MappedIterator<'p, Enumerator<'p>, Path>;

impl<'u> Enumerator<'u> {
    /// Get the udev context.
    pub fn udev(&self) -> &Udev {
        self.udev
    }

    /// Include devices with the specified subsystem.
    ///
    /// All devices added by future scans will match either the specified subsystem or a subsystem
    /// specified in a previous invocation of this function (on this enumerator). If this function
    /// has never been called, devices will not be filtered by subsystem.
    pub fn match_subsystem(self, subsystem: &str) -> Enumerator<'u> {
        subsystem.with_c_str(|subsystem| util::handle_error(unsafe {
            libudev_c::udev_enumerate_add_match_subsystem(self.enumerator, subsystem)
        }));
        self
    }

    /// Exclude devices with the specified subsystem.
    ///
    /// No device added by a future scan will have the specified subsystem.
    pub fn match_not_subsystem(self, subsystem: &str) -> Enumerator<'u> {
        subsystem.with_c_str(|subsystem| util::handle_error(unsafe {
            libudev_c::udev_enumerate_add_nomatch_subsystem(self.enumerator, subsystem)
        }));
        self
    }

    /// Only include devices with an attribute.
    ///
    /// All devices added by future scans will have the specified attribute with
    /// the (optionally) specified value.
    pub fn match_attribute(self, attr: &str, value: Option<&str>) -> Enumerator<'u> {
        fn it(e: &Enumerator, attr: *const i8, value: *const i8) {
            util::handle_error(unsafe {libudev_c::udev_enumerate_add_match_sysattr(e.enumerator, attr, value)});
        }
        attr.with_c_str(|attr| match value {
            Some(value) => value.with_c_str(|value| it(&self, attr, value)),
            None => it(&self, attr, ptr::null())
        });
        self
    }

    /// Exclude devices with an attribute.
    ///
    /// No device added by future scans will have the specified attribute with
    /// the (optionally) specified value.
    pub fn match_not_attribute(self, attr: &str, value: Option<&str>) -> Enumerator<'u> {
        fn it(e: &Enumerator, attr: *const i8, value: *const i8) {
            util::handle_error(unsafe {libudev_c::udev_enumerate_add_nomatch_sysattr(e.enumerator, attr, value)});
        }
        attr.with_c_str(|attr| match value {
            Some(value) => value.with_c_str(|value| it(&self, attr, value)),
            None => it(&self, attr, ptr::null())
        });
        self
    }

    /// Only include devices with a property.
    ///
    /// All devices added by future scans will have the specified property with
    /// the (optionally) specified value.
    pub fn match_property(self, attr: &str, value: Option<&str>) -> Enumerator<'u> {
        fn it(e: &Enumerator, attr: *const i8, value: *const i8) {
            util::handle_error(unsafe {libudev_c::udev_enumerate_add_match_property(e.enumerator, attr, value)});
        }
        attr.with_c_str(|attr| match value {
            Some(value) => value.with_c_str(|value| it(&self, attr, value)),
            None => it(&self, attr, ptr::null())
        });
        self
    }

    /// Only include the specified device and its children.
    ///
    /// All devices added by future scans (until the parent is changed/cleared)
    /// will have/be the specified parent.
    pub fn match_parent(self, parent: &Device) -> Enumerator<'u> {
        unsafe {
            util::handle_error(libudev_c::udev_enumerate_add_match_parent(self.enumerator, device::device_get_dev(parent)));
        }
        self
    }

    /// Remove the parent restriction.
    ///
    /// Clear the current parent match. Future scans will add devices regardless of their parents.
    pub fn clear_parent(self) -> Enumerator<'u> {
        unsafe {
            util::handle_error(libudev_c::udev_enumerate_add_match_parent(self.enumerator, ptr::null_mut()));
        }
        self
    }

    /// Only include devices with the specified tag.
    ///
    /// All devices added by future scans will match the specified tag.
    pub fn match_tag(self, tag: &str) -> Enumerator<'u> {
        tag.with_c_str(|tag| util::handle_error( unsafe {
            libudev_c::udev_enumerate_add_match_tag(self.enumerator, tag)
        }));
        self
    }

    /// Include only initialized devices.
    ///
    /// All devices added by future scans will be initialized.
    pub fn match_is_initialized(self) -> Enumerator<'u> {
        util::handle_error(unsafe {
            libudev_c::udev_enumerate_add_match_is_initialized(self.enumerator)
        });
        self
    }

    /// Include devices with the specified sysname.
    ///
    /// All devices added by future scans will match either the specified sysname or a sysname
    /// specified in a previous invocation of this function (on this enumerator). If this function
    /// has never been called, devices will not be filtered by sysname.
    pub fn match_sysname(self, sysname: &str) -> Enumerator<'u> {
        sysname.with_c_str(|sysname| util::handle_error( unsafe {
            libudev_c::udev_enumerate_add_match_sysname(self.enumerator, sysname)
        }));
        self
    }

    /// Manually include a device.
    ///
    /// Manually add a device to the enumerator bypassing matches. According to the libudev
    /// documentation, this can be useful for determine device dependency order (see iter below).
    pub fn add_device(self, device: &Device) -> Enumerator<'u> {
        device.syspath().with_c_str(|syspath| util::handle_error(unsafe{
            libudev_c::udev_enumerate_add_syspath(self.enumerator, syspath)
        }));
        self
    }

    /// Scan subsystems
    ///
    /// Scan sysfs for subsystems matching all previously applied constraints and add them to the
    /// enumerator.
    pub fn scan_subsystems(self) -> Enumerator<'u> {
        util::handle_error(unsafe { libudev_c::udev_enumerate_scan_subsystems(self.enumerator) });
        self
    }

    /// Scan devices
    ///
    /// Scan sysfs for devices matching all previously applied constraints and add them to the
    /// enumerator.
    pub fn scan_devices(self) -> Enumerator<'u> {
        util::handle_error(unsafe { libudev_c::udev_enumerate_scan_devices(self.enumerator) });
        self
    }

    /// Iterate over all devices that have been added to this iterator in dependency order.
    ///
    /// # Note
    ///
    /// Enumerators are not prepopulated with devices. If you haven't either manually added devices
    /// to the enumerator (`add_device`), or haven't called a scan method (`scan_subsystems` or
    /// `scan_devices`), this iterator will be empty.
    pub fn iter(&self) -> DeviceIterator {
        unsafe {
            iterator::iterator(self, libudev_c::udev_enumerate_get_list_entry(self.enumerator))
        }.filter_map(|(this, key, _)| this.udev.device(&Path::new(key)))
    }

    /// Same as `iter` but avoid creating device objects.
    pub fn iter_paths(&self) -> DevicePathIterator {
        unsafe {
            iterator::iterator(self, libudev_c::udev_enumerate_get_list_entry(self.enumerator))
        }.map(|(_, key, _)| Path::new(key))
    }

}

#[unsafe_destructor]
impl<'p> Drop for Enumerator<'p> {
    fn drop(&mut self) {
        unsafe {
            libudev_c::udev_enumerate_unref(self.enumerator);
        }
    }
}
