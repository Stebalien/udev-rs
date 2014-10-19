use std::ptr;
use std::from_str::FromStr;
use libc::{fcntl, O_NONBLOCK, F_SETFL, F_GETFL, ENOMEM, EINVAL};
use std::io::IoError;

use device;
use libudev_c;
use util;

use udev::Udev;
use device::Device;


pub struct Monitor<'u> {
    pub udev: &'u Udev,
    monitor: libudev_c::udev_monitor
}

#[deriving(Show)]
pub enum Action {
    Add,
    Remove,
    Change,
    Move,
    Online,
    Offline
}

#[deriving(Show)]
pub struct Event {
    action: Action,
    seqnum: u64
}

macro_rules! handle_ioerror( () => (
        return match util::get_errno() {
            ENOMEM | EINVAL => fail!("BUG"),
            e => Err(IoError::from_errno(e as uint, true))
        }
))

pub struct MonitorIterator<'m, 'u: 'm> {
    monitor: &'m Monitor<'u>
}

pub fn monitor<'u>(udev: &'u Udev, f: || -> libudev_c::udev_monitor) -> Result<Monitor<'u>, IoError> {
    let monitor = match util::check_errno(f) {
        Ok(Some(monitor))        => monitor,
        Err(EINVAL) | Ok(None)  => fail!("BUG"),
        Err(e)                  => return Err(IoError::from_errno(e as uint, true))
    };
    let fd = unsafe {
        libudev_c::udev_monitor_get_fd(monitor)
    };

    let old_val = unsafe { fcntl(fd, F_GETFL) };
    if old_val == -1 {
        handle_ioerror!();
    }

    if unsafe { fcntl(fd, F_SETFL, old_val & !O_NONBLOCK) == -1 } {
        handle_ioerror!();
    }

    Ok(Monitor {
        udev: udev,
        monitor: monitor
    })
}

#[allow(unused_mut)]
impl<'u> Monitor<'u> {
    pub fn filter_subsystem(mut self, subsystem: &str) -> Monitor<'u> {
        subsystem.with_c_str(|subsystem| util::handle_error(unsafe {
            libudev_c::udev_monitor_filter_add_match_subsystem_devtype(self.monitor, subsystem, ptr::null())
        }));
        self
    }
    pub fn filter_subsystem_devtype(mut self, subsystem: &str, devtype: &str) -> Monitor<'u> {
        subsystem.with_c_str(|subsystem| devtype.with_c_str(|devtype| util::handle_error(unsafe {
            libudev_c::udev_monitor_filter_add_match_subsystem_devtype(self.monitor, subsystem, devtype)
        })));
        self
    }
    pub fn filter_tag(mut self, tag: &str) -> Monitor<'u> {
        tag.with_c_str(|tag| util::handle_error(unsafe {
            libudev_c::udev_monitor_filter_add_match_tag(self.monitor, tag)
        }));
        self
    }
    pub fn unfilter(mut self) -> Monitor<'u> {
        util::handle_error(unsafe {
            libudev_c::udev_monitor_filter_remove(self.monitor)
        });
        self
    }
    pub fn iter<'m>(&'m self) -> MonitorIterator<'m, 'u> {
        util::handle_error(unsafe {
            // Technically this mutates but we're single threaded anyways. Basically, having two
            // iterators existing at the same time won't cause any problems because next() can't be
            // called at the same time (single threaded).
            libudev_c::udev_monitor_enable_receiving(self.monitor)
        });
        MonitorIterator::<'m, 'u> {
            monitor: self
        }
    }
}

#[unsafe_destructor]
impl<'u> Drop for Monitor<'u> {
    fn drop(&mut self) {
        unsafe {
            libudev_c::udev_monitor_unref(self.monitor);
        }
    }
}

impl FromStr for Action {
    fn from_str(s: &str) -> Option<Action> {
        match s {
            "add"       => Some(Add),
            "remove"    => Some(Remove),
            "change"    => Some(Change),
            "move"      => Some(Move),
            "online"    => Some(Online),
            "offline"   => Some(Offline),
            _           => None
        }
    }
}

impl<'m, 'u> Iterator<(Event, Device<'u>)> for MonitorIterator<'m, 'u> {
    fn next(&mut self) -> Option<(Event, Device<'u>)> {
        loop {
            if let Ok(Some(dev)) = util::check_errno(|| unsafe {
                libudev_c::udev_monitor_receive_device(self.monitor.monitor)
            }) {
                return Some((
                    Event {
                        action: from_str(unsafe {
                                    util::c_to_str(libudev_c::udev_device_get_action(dev))
                                }.unwrap()).unwrap(),
                        seqnum: unsafe {
                                    libudev_c::udev_device_get_seqnum(dev)
                                }
                    },
                    unsafe { device::device(self.monitor.udev, dev) }
                ));
            }
        }
    }
}

