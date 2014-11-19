use std::ptr;
use std::io::{IoError, standard_error, FileNotFound};
use std::fmt;
use libc::dev_t;
use std::time::Duration;

use udev::{
    libudev_c,
    util,
    iterator,
};
    
use udev::udev::Udev;
use udev::iterator::MappedIterator;

pub struct Device<'u> {
    udev: &'u Udev,
    dev: libudev_c::udev_device,
}

#[doc(hidden)]
pub type TagIterator<'p> = MappedIterator<'p, Device<'p>, &'p str>;
#[doc(hidden)]
pub type AttributeIterator<'p> = MappedIterator<'p, Device<'p>, &'p str>;
#[doc(hidden)]
pub type DevlinkIterator<'p> = MappedIterator<'p, Device<'p>, Path>;
#[doc(hidden)]
pub type PropertyIterator<'p> = MappedIterator<'p, Device<'p>, (&'p str, Option<&'p str>)>;

pub type Devnum = dev_t;
pub enum Type {
    Char,
    Block
}

// Crate Private
pub unsafe fn device<'u>(udev: &'u Udev, dev: libudev_c::udev_device) -> Device<'u> {
    Device { udev: udev, dev: dev }
}

pub unsafe fn device_get_dev(device: &Device) -> libudev_c::udev_device {
    device.dev
}

impl<'u> Device<'u> {
    /// Get the udev context.
    pub fn udev(&self) -> &Udev {
        self.udev
    }

    /// Get the device's parent if one exists.
    pub fn parent(&self) -> Option<Device> {
        match util::check_errno(|| unsafe {
            libudev_c::udev_device_ref(libudev_c::udev_device_get_parent(self.dev))
        }) {
            Ok(Some(dev)) => Some(unsafe { device(self.udev, dev) }),
            _ => None
        }
    }

    /// Get the first parent with the specified subsystem.
    pub fn parent_with_subsystem(&self, subsystem: &str) -> Option<Device> {
        match subsystem.with_c_str(|subsystem| util::check_errno(|| unsafe {
            libudev_c::udev_device_ref(
                libudev_c::udev_device_get_parent_with_subsystem_devtype(self.dev, subsystem, ptr::null()))
        })) {
            Ok(Some(dev)) => Some(unsafe { device(self.udev, dev) }),
            _ => None
        }
    }

    /// Get the first parent with the specified subsystem and devtype.
    pub fn parent_with_subsystem_devtype(&self, subsystem: &str, devtype: &str) -> Option<Device> {
        match subsystem.with_c_str(|subsystem| devtype.with_c_str(|devtype| util::check_errno(|| unsafe {
            libudev_c::udev_device_ref(
                libudev_c::udev_device_get_parent_with_subsystem_devtype(self.dev, subsystem, devtype))
        }))) {
            Ok(Some(dev)) => Some(unsafe { device(self.udev, dev) }),
            _ => None
        }
    }

    /// Read a sysfs attribute.
    pub fn attribute<'s>(&'s self, attr: &str) -> Result<&'s str, IoError> {
        match attr.with_c_str(|cstr| util::check_errno(|| unsafe {
            libudev_c::udev_device_get_sysattr_value(self.dev, cstr)
        })) {
            Ok(Some(val)) => Ok(unsafe { util::c_to_str(val) }.unwrap()),
            Ok(None) => Err(standard_error(FileNotFound)),
            Err(errno) => Err(IoError::from_errno(errno as uint, true)),
        }
    }

    /// Write a sysfs attribute.
    pub fn set_attribute(&self, attr: &str, value: &str) -> Result<(), IoError> {
        attr.with_c_str(|attr| value.with_c_str(|value| match unsafe {
            libudev_c::udev_device_set_sysattr_value(self.dev, attr, value)
        } {
            0           => Ok(()),
            n if n < 0  => Err(IoError::from_errno(-n as uint, true)),
            _           => panic!("udev returned an invalid error")
        }))
    }

    /// Get the path to the device (minus `/sys`).
    pub fn devpath<'s>(&'s self) -> &'s str {
        unsafe {
            util::c_to_str(libudev_c::udev_device_get_devpath(self.dev)).unwrap()
        }
    }

    /// Get the full path to the device (including `/sys`).
    pub fn syspath<'s>(&'s self) -> Path {
        unsafe {
            Path::new(util::c_to_str(libudev_c::udev_device_get_syspath(self.dev)).unwrap())
        }
    }

    /// Get the device name.
    ///
    /// E.g. wlan0
    pub fn sysname<'s>(&'s self) -> &'s str {
        unsafe {
            util::c_to_str(libudev_c::udev_device_get_sysname(self.dev)).unwrap()
        }
    }

    /// Get the devices subsystem
    pub fn subsystem<'s>(&'s self) -> Option<&'s str> {
        unsafe {
            util::c_to_str(libudev_c::udev_device_get_subsystem(self.dev))
        }
    }

    /// Get the devices devtype
    pub fn devtype<'s>(&'s self) -> Option<&'s str> {
        unsafe {
            util::c_to_str(libudev_c::udev_device_get_devtype(self.dev))
        }
    }

    /// Get the devices sysnum.
    ///
    /// E.g. the X in ethX, wlanX, etc.
    pub fn sysnum(&self) -> Option<u64> {
        match unsafe {
            util::c_to_str(libudev_c::udev_device_get_sysnum(self.dev))
        } {
            Some(n) => from_str(n),
            None => None
        }
    }

    /// Get the device's devnum.
    pub fn devnum(&self) -> Option<Devnum> {
        match unsafe {
            libudev_c::udev_device_get_devnum(self.dev)
        } {
            0 => None,
            n => Some(n)
        }
    }

    /// Get the device's driver.
    pub fn driver(&self) -> Option<&str> {
        unsafe {
            util::c_to_str(libudev_c::udev_device_get_driver(self.dev))
        }
    }

    /// Get the device's devnode
    ///
    /// E.g. `/dev/sda`
    pub fn devnode(& self) -> Option<Path> {
        unsafe {
            util::c_to_str(libudev_c::udev_device_get_devnode(self.dev))
        }.map(|path| Path::new(path))
    }

    /// Iterate over the device's devlinks
    ///
    /// E.g. the symlinks in `/dev/disk/by-*/`
    pub fn iter_devlinks(&self) -> DevlinkIterator {
        unsafe {
            iterator::iterator(self, libudev_c::udev_device_get_devlinks_list_entry(self.dev))
        }.map(|(_, key, _)| Path::new(key))
    }

    /// Iterate over the device's tags.
    pub fn iter_tags(&self) -> TagIterator {
        unsafe {
            iterator::iterator(self, libudev_c::udev_device_get_tags_list_entry(self.dev))
        }.map(|(_, key, _)| key)
    }

    /// Iterate over the device's properties.
    pub fn iter_properties(&self) -> PropertyIterator {
        unsafe {
            iterator::iterator(self, libudev_c::udev_device_get_properties_list_entry(self.dev))
        }.map(|(_, key, value)| (key, value))
    }

    /// Iterate over the device's sysfs attribute names
    pub fn iter_attributes(& self) -> AttributeIterator {
        unsafe {
            iterator::iterator(self, libudev_c::udev_device_get_sysattr_list_entry(self.dev))
        }.map(|(_, key, _)| key)
    }

    /// Get the time since the device was initialized by udev.
    pub fn time_since_initialized(&self) -> Option<Duration> {
        let usec = unsafe { libudev_c::udev_device_get_usec_since_initialized(self.dev) };
        if usec == 0 {
            None
        } else {
            // Note: I don't support machines that are online for over 292,471 years. Sorry.
            Some(Duration::microseconds(usec as i64))
        }
    }

    /// Determine if the device has been initialized.
    pub fn is_initialized(&self) -> bool {
        unsafe { libudev_c::udev_device_get_is_initialized(self.dev) != 0 }
    }

    /// Check whether the device is tagged with a given tag.
    pub fn has_tag(&self, tag: &str) -> bool {
        tag.with_c_str(|tag| unsafe {
            libudev_c::udev_device_has_tag(self.dev, tag) != 0
        })
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

impl Type {
    pub fn to_char(self) -> i8 {
        match self {
            Type::Char => 'c' as i8,
            Type::Block => 'b' as i8
        }
    }
}
