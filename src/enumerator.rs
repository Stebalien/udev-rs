use std::ptr;
use std::iter;

use udev;
use libudev_c;
use util;

use udev::Udev;
use device::Device;
use iterator::UdevIterator;

pub struct DeviceEnumerator<'u> {
    pub udev: &'u Udev,
    enumerator: libudev_c::udev_enumerate
}

pub struct SubsystemEnumerator<'u> {
    pub udev: &'u Udev,
    enumerator: libudev_c::udev_enumerate
}

unsafe fn new_enumerator<'u>(udev: &'u Udev) -> libudev_c::udev_enumerate {
    // Check errno to check for ENOMEM
    util::check_errno(|| {
            libudev_c::udev_enumerate_new(udev::get_udev_ctx(udev))
    }).unwrap().unwrap()
}

// Crate Private
pub fn device_enumerator<'u>(udev: &'u Udev, parent: Option<libudev_c::udev_device>) -> DeviceEnumerator<'u> {
    // Check errno to check for ENOMEM
    let enumerator = unsafe { new_enumerator(udev) };

    if let Some(parent) = parent {
        unsafe { util::handle_error(libudev_c::udev_enumerate_add_match_parent(enumerator, parent)); }
    }

    DeviceEnumerator {
        udev: udev,
        enumerator: enumerator
    }
}

// Crate Private
pub fn subsystem_enumerator<'u>(udev: &'u Udev) -> SubsystemEnumerator<'u> {
    SubsystemEnumerator {
        udev: udev,
        enumerator: unsafe { new_enumerator(udev) }
    }
}


#[allow(unused_mut)]
impl<'u> DeviceEnumerator<'u> {
    pub fn filter_subsystem(mut self, subsystem: &str) -> DeviceEnumerator<'u> {
        subsystem.with_c_str(|subsystem| util::handle_error(unsafe {
            libudev_c::udev_enumerate_add_match_subsystem(self.enumerator, subsystem)
        }));
        self
    }
    pub fn filter_not_subsystem(mut self, subsystem: &str) -> DeviceEnumerator<'u> {
        subsystem.with_c_str(|subsystem| util::handle_error(unsafe {
            libudev_c::udev_enumerate_add_nomatch_subsystem(self.enumerator, subsystem)
        }));
        self
    }
    pub fn filter_attribute(mut self, attr: &str, value: Option<&str>) -> DeviceEnumerator<'u> {
        fn it(e: &DeviceEnumerator, attr: *const i8, value: *const i8) {
            util::handle_error(unsafe {libudev_c::udev_enumerate_add_match_sysattr(e.enumerator, attr, value)});
        }
        attr.with_c_str(|attr| match value {
            Some(value) => value.with_c_str(|value| it(&self, attr, value)),
            None => it(&self, attr, ptr::null())
        });
        self
    }

    pub fn filter_not_attribute(mut self, attr: &str, value: Option<&str>) -> DeviceEnumerator<'u> {
        fn it(e: &DeviceEnumerator, attr: *const i8, value: *const i8) {
            util::handle_error(unsafe {libudev_c::udev_enumerate_add_nomatch_sysattr(e.enumerator, attr, value)});
        }
        attr.with_c_str(|attr| match value {
            Some(value) => value.with_c_str(|value| it(&self, attr, value)),
            None => it(&self, attr, ptr::null())
        });
        self
    }

    pub fn filter_property(mut self, attr: &str, value: Option<&str>) -> DeviceEnumerator<'u> {
        fn it(e: &DeviceEnumerator, attr: *const i8, value: *const i8) {
            util::handle_error(unsafe {libudev_c::udev_enumerate_add_match_property(e.enumerator, attr, value)});
        }
        attr.with_c_str(|attr| match value {
            Some(value) => value.with_c_str(|value| it(&self, attr, value)),
            None => it(&self, attr, ptr::null())
        });
        self
    }

    pub fn filter_tag(mut self, tag: &str) -> DeviceEnumerator<'u> {
        tag.with_c_str(|tag| util::handle_error( unsafe {
            libudev_c::udev_enumerate_add_match_tag(self.enumerator, tag)
        }));
        self
    }

    pub fn filter_initialized(mut self) -> DeviceEnumerator<'u> {
        util::handle_error(unsafe {
            libudev_c::udev_enumerate_add_match_is_initialized(self.enumerator)
        });
        self
    }

    pub fn filter_sysname(mut self, sysname: &str) -> DeviceEnumerator<'u> {
        sysname.with_c_str(|sysname| util::handle_error( unsafe {
            libudev_c::udev_enumerate_add_match_sysname(self.enumerator, sysname)
        }));
        self
    }

    pub fn include_device(mut self, device: &Device) -> DeviceEnumerator<'u> {
        device.syspath().with_c_str(|syspath| util::handle_error(unsafe{
            libudev_c::udev_enumerate_add_syspath(self.enumerator, syspath)
        }));
        self
    }

    fn make_iter(&mut self) -> UdevIterator<DeviceEnumerator> {
        util::handle_error(unsafe { libudev_c::udev_enumerate_scan_devices(self.enumerator) });
        UdevIterator::new(self, unsafe{
            libudev_c::udev_enumerate_get_list_entry(self.enumerator)
        })
    }

    pub fn iter_paths(&mut self) -> iter::Map<(&DeviceEnumerator, &str, Option<&str>), Path, UdevIterator<DeviceEnumerator>> {
        self.make_iter().map(|(_, key, _)| Path::new(key))
    }

    pub fn iter(&mut self) -> iter::FilterMap<(&DeviceEnumerator, &str, Option<&str>), Device, UdevIterator<DeviceEnumerator>> {
        self.make_iter().filter_map(|(this, key, _)| this.udev.device(&Path::new(key)))
    }
}

#[unsafe_destructor]
impl<'p> Drop for DeviceEnumerator<'p> {
    fn drop(&mut self) {
        unsafe {
            libudev_c::udev_enumerate_unref(self.enumerator);
        }
    }
}

impl<'p> SubsystemEnumerator<'p> {
    pub fn iter(&mut self) -> iter::Map<(&SubsystemEnumerator, &str, Option<&str>), &str, UdevIterator<SubsystemEnumerator>> {
        util::handle_error(unsafe { libudev_c::udev_enumerate_scan_subsystems(self.enumerator) });
        UdevIterator::new(self, unsafe {
            libudev_c::udev_enumerate_get_list_entry(self.enumerator)
        }).map(|(_, key, _)| key)
    }
}

#[unsafe_destructor]
impl<'p> Drop for SubsystemEnumerator<'p> {
    fn drop(&mut self) {
        unsafe {
            libudev_c::udev_enumerate_unref(self.enumerator);
        }
    }
}

