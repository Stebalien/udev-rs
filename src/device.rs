use std::ptr;
use std::io::IoError;
use std::fmt;
use libc::dev_t;
use std::iter;
use std::time::Duration;

use libudev_c;
use util;
use enumerator;

use udev::Udev;
use enumerator::DeviceEnumerator;
use iterator::UdevIterator;

pub struct Device<'u> {
    pub udev: &'u Udev,
    dev: libudev_c::udev_device,
}
pub type Devnum = dev_t;
pub enum DeviceType {
    CharDev,
    BlockDev
}

pub unsafe fn device<'u>(udev: &'u Udev, dev: libudev_c::udev_device, doref: bool) -> Option<Device<'u>> {
    if dev.is_null() {
        // TODO Check oom/errno
        None
    } else {
        if doref { 
            libudev_c::udev_device_ref(dev);
        }
        Some(Device { udev: udev, dev: dev })
    }
}

impl<'u> Device<'u> {
    pub fn parent(&self) -> Option<Device> {
        unsafe {
            device(
                self.udev, 
                libudev_c::udev_device_get_parent(self.dev),
                true)
        }
    }

    pub fn parent_with_subsystem(&self, subsystem: &str) -> Option<Device> {
        subsystem.with_c_str(|subsystem| unsafe {
            device(
                self.udev, 
                libudev_c::udev_device_get_parent_with_subsystem_devtype(self.dev, subsystem, ptr::null()),
                true)
        })
    }

    pub fn parent_with_subsystem_devtype(&self, subsystem: &str, devtype: &str) -> Option<Device> {
        subsystem.with_c_str(|subsystem| devtype.with_c_str(|devtype| unsafe {
            device(
                self.udev, 
                libudev_c::udev_device_get_parent_with_subsystem_devtype(self.dev, subsystem, devtype),
                true)
        }))
    }

    pub fn attribute<'s>(&'s self, attr: &str) -> Option<&'s str> {
        attr.with_c_str(|cstr| unsafe {
            util::c_to_str(libudev_c::udev_device_get_sysattr_value(self.dev, cstr))
        })
    }

    pub fn set_attribute(&self, attr: &str, value: &str) -> Result<(), IoError> {
        attr.with_c_str(|attr| value.with_c_str(|value| match unsafe {
            libudev_c::udev_device_set_sysattr_value(self.dev, attr, value)
        } {
            0           => Ok(()),
            n if n < 0  => Err(IoError::from_errno(-n as uint, true)),
            _           => fail!("udev returned an invalid error")
        }))
    }

    pub fn devpath<'s>(&'s self) -> &'s str {
        unsafe {
            util::c_to_str(libudev_c::udev_device_get_devpath(self.dev)).unwrap()
        }
    }

    pub fn syspath<'s>(&'s self) -> Path {
        unsafe {
            Path::new(util::c_to_str(libudev_c::udev_device_get_syspath(self.dev)).unwrap())
        }
    }

    pub fn sysname<'s>(&'s self) -> &'s str {
        unsafe {
            util::c_to_str(libudev_c::udev_device_get_sysname(self.dev)).unwrap()
        }
    }


    pub fn subsystem<'s>(&'s self) -> Option<&'s str> {
        unsafe {
            util::c_to_str(libudev_c::udev_device_get_subsystem(self.dev))
        }
    }

    pub fn devtype<'s>(&'s self) -> Option<&'s str> {
        unsafe {
            util::c_to_str(libudev_c::udev_device_get_devtype(self.dev))
        }
    }

    pub fn sysnum(&self) -> Option<uint> {
        match unsafe {
            util::c_to_str(libudev_c::udev_device_get_sysnum(self.dev))
        } {
            Some(n) => from_str(n),
            None => None
        }
    }

    pub fn devnum(&self) -> Devnum {
        unsafe {
            libudev_c::udev_device_get_devnum(self.dev)
        }
    }

    pub fn driver<'s>(&'s self) -> Option<&'s str> {
        unsafe {
            util::c_to_str(libudev_c::udev_device_get_driver(self.dev))
        }
    }

    pub fn devnode<'s>(&'s self) -> Option<&'s str> {
        unsafe {
            util::c_to_str(libudev_c::udev_device_get_devnode(self.dev))
        }
    }

    pub fn iter_devlinks(&self) -> iter::Map<(&Device,&str,Option<&str>),Path,UdevIterator<Device>> {
        UdevIterator::new(self, unsafe {
            libudev_c::udev_device_get_devlinks_list_entry(self.dev)
        }).map(|(_, key, _)| Path::new(key))
    }

    pub fn iter_tags(&self) -> iter::Map<(&Device,&str,Option<&str>),&str,UdevIterator<Device>> {
        UdevIterator::new(self, unsafe {
            libudev_c::udev_device_get_tags_list_entry(self.dev)
        }).map(|(_, key, _)| key)
    }

    pub fn iter_properties(&self) -> iter::Map<(&Device,&str,Option<&str>),(&str, Option<&str>),UdevIterator<Device>> {
        UdevIterator::new(self, unsafe {
            libudev_c::udev_device_get_properties_list_entry(self.dev)
        }).map(|(_, key, value)| (key, value))
    }

    pub fn iter_attributes(& self) -> iter::Map<(&Device,&str,Option<&str>),&str,UdevIterator<Device>> {
        UdevIterator::new(self, unsafe {
            libudev_c::udev_device_get_sysattr_list_entry(self.dev)
        }).map(|(_, key, _)| key)
    }

    pub fn time_since_initialized(&self) -> Option<Duration> {
        let usec = unsafe { libudev_c::udev_device_get_usec_since_initialized(self.dev) };
        if usec == 0 {
            None
        } else {
            // Note: I don't support machines that are online for over 292,471 years. Sorry.
            Some(Duration::microseconds(usec as i64))
        }
    }

    pub fn is_initialized(&self) -> bool {
        unsafe { libudev_c::udev_device_get_is_initialized(self.dev) != 0 }
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        tag.with_c_str(|tag| unsafe {
            libudev_c::udev_device_has_tag(self.dev, tag) != 0
        })
    }

    pub fn children(&self) -> DeviceEnumerator<'u> {
        unsafe {
            enumerator::device_enumerator(self.udev, Some(self.dev))
        }
    }
}

#[unsafe_destructor]
impl<'u> Drop for Device<'u> {
    fn drop(&mut self) {
        unsafe { libudev_c::udev_device_unref(self.dev) };
    }
}

impl<'u> fmt::Show for Device<'u> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.syspath().as_str().unwrap())
    }
}

impl DeviceType {
    pub fn to_char(self) -> i8 {
        match self {
            CharDev => 'c' as i8,
            BlockDev => 'b' as i8
        }
    }
}
