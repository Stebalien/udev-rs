use std::ptr;
use std::iter;

use libudev_c;
use util;
use iterator;
use device;

use udev::Udev;
use device::Device;
use iterator::UdevIterator;

pub struct Enumerator<'u> {
    pub udev: &'u Udev,
    enumerator: libudev_c::udev_enumerate
}

// Crate Private
pub unsafe fn enumerator<'u>(udev: &'u Udev, enumerator: libudev_c::udev_enumerate) -> Enumerator<'u> {
    Enumerator {
        udev: udev,
        enumerator: enumerator
    }
}

impl<'u> Enumerator<'u> {
    pub fn match_subsystem(self, subsystem: &str) -> Enumerator<'u> {
        subsystem.with_c_str(|subsystem| util::handle_error(unsafe {
            libudev_c::udev_enumerate_add_match_subsystem(self.enumerator, subsystem)
        }));
        self
    }
    pub fn match_not_subsystem(self, subsystem: &str) -> Enumerator<'u> {
        subsystem.with_c_str(|subsystem| util::handle_error(unsafe {
            libudev_c::udev_enumerate_add_nomatch_subsystem(self.enumerator, subsystem)
        }));
        self
    }
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

    pub fn match_parent(self, parent: &Device) -> Enumerator<'u> {
        unsafe {
            util::handle_error(libudev_c::udev_enumerate_add_match_parent(self.enumerator, device::device_get_dev(parent)));
        }
        self
    }

    pub fn clear_parent(self) -> Enumerator<'u> {
        unsafe {
            util::handle_error(libudev_c::udev_enumerate_add_match_parent(self.enumerator, ptr::null_mut()));
        }
        self
    }

    pub fn match_tag(self, tag: &str) -> Enumerator<'u> {
        tag.with_c_str(|tag| util::handle_error( unsafe {
            libudev_c::udev_enumerate_add_match_tag(self.enumerator, tag)
        }));
        self
    }

    pub fn match_is_initialized(self) -> Enumerator<'u> {
        util::handle_error(unsafe {
            libudev_c::udev_enumerate_add_match_is_initialized(self.enumerator)
        });
        self
    }

    pub fn match_sysname(self, sysname: &str) -> Enumerator<'u> {
        sysname.with_c_str(|sysname| util::handle_error( unsafe {
            libudev_c::udev_enumerate_add_match_sysname(self.enumerator, sysname)
        }));
        self
    }

    pub fn add_device(self, device: &Device) -> Enumerator<'u> {
        device.syspath().with_c_str(|syspath| util::handle_error(unsafe{
            libudev_c::udev_enumerate_add_syspath(self.enumerator, syspath)
        }));
        self
    }

    pub fn scan_subsystems(self) -> Enumerator<'u> {
        util::handle_error(unsafe { libudev_c::udev_enumerate_scan_subsystems(self.enumerator) });
        self
    }

    pub fn scan_devices(self) -> Enumerator<'u> {
        util::handle_error(unsafe { libudev_c::udev_enumerate_scan_devices(self.enumerator) });
        self
    }

    fn make_iter(&self) -> UdevIterator<Enumerator> {
        unsafe {
            iterator::udev_iterator(self, libudev_c::udev_enumerate_get_list_entry(self.enumerator))
        }
    }

    pub fn iter_paths(&self) -> iter::Map<(&Enumerator, &str, Option<&str>), Path, UdevIterator<Enumerator>> {
        self.make_iter().map(|(_, key, _)| Path::new(key))
    }

    pub fn iter(&self) -> iter::FilterMap<(&Enumerator, &str, Option<&str>), Device, UdevIterator<Enumerator>> {
        self.make_iter().filter_map(|(this, key, _)| this.udev.device(&Path::new(key)))
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
