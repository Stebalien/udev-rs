use std::ptr;
use std::from_str::FromStr;

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
    pub action: Action,
    pub seqnum: u64
}

pub struct MonitorIterator<'m, 'u: 'm> {
    monitor: &'m Monitor<'u>
}

pub unsafe fn monitor(udev: &Udev, monitor: libudev_c::udev_monitor) -> Monitor {
    Monitor {
        udev: udev,
        monitor: monitor
    }
}

impl<'u> Monitor<'u> {
    /// Filter by subsystem.
    ///
    /// Exclude devices that don't match the specified subsystem or a previously specified
    /// subsystem.
    pub fn filter_subsystem(self, subsystem: &str) -> Monitor<'u> {
        subsystem.with_c_str(|subsystem| util::handle_error(unsafe {
            libudev_c::udev_monitor_filter_add_match_subsystem_devtype(self.monitor, subsystem, ptr::null())
        }));
        self
    }
    /// Filter by subsystem/devtype combination.
    ///
    /// Exclude devices that don't match the specified subsystem/devtype combination or a
    /// previously specified subsystem/devtype combination (or any subsystem previously specified
    /// in a `filter_subsystem` invocation).
    pub fn filter_subsystem_devtype(self, subsystem: &str, devtype: &str) -> Monitor<'u> {
        subsystem.with_c_str(|subsystem| devtype.with_c_str(|devtype| util::handle_error(unsafe {
            libudev_c::udev_monitor_filter_add_match_subsystem_devtype(self.monitor, subsystem, devtype)
        })));
        self
    }
    /// Filter by tag.
    ///
    /// Exclude devices that don't match the specified tag or a previously specified tag.
    pub fn filter_tag(self, tag: &str) -> Monitor<'u> {
        tag.with_c_str(|tag| util::handle_error(unsafe {
            libudev_c::udev_monitor_filter_add_match_tag(self.monitor, tag)
        }));
        self
    }

    /// Reset all filters on this monitor. No devices will be excluded.
    pub fn unfilter(self) -> Monitor<'u> {
        util::handle_error(unsafe {
            libudev_c::udev_monitor_filter_remove(self.monitor)
        });
        self
    }

    /// Iterate over udev events.
    ///
    /// 1. The returned iterator will block on calls to next until their a device is available.
    /// 2. The returned iterator will never end (next will never return None).
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

