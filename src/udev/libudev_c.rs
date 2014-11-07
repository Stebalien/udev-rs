#![allow(non_camel_case_types)]

use libc::{c_void, c_char, c_int, dev_t};

pub type udev = *mut c_void;
pub type udev_device = *mut c_void;
pub type udev_list_entry = *mut c_void;
pub type udev_enumerate = *mut c_void;
pub type udev_monitor = *mut c_void;
pub type udev_hwdb = *mut c_void;
type c_str = *const c_char;

#[link(name = "udev")]
extern {
    // Udev
    pub fn udev_new() -> udev;
    pub fn udev_unref(udev: udev) -> udev;

    // Device
    pub fn udev_device_new_from_syspath(udev: udev, attr: c_str) -> udev_device;
    pub fn udev_device_new_from_devnum(dev: udev_device, c: c_char, d: dev_t) -> udev_device;
    pub fn udev_device_new_from_subsystem_sysname(dev: udev_device, ss: c_str, sn: c_str) -> udev_device;
    pub fn udev_device_unref(dev: udev_device) -> udev_device;
    pub fn udev_device_ref(dev: udev_device) -> udev_device;

    pub fn udev_device_get_parent(dev: udev_device) -> udev_device;
    pub fn udev_device_get_parent_with_subsystem_devtype(dev: udev_device, sub: c_str, ty: c_str) -> udev_device;

    pub fn udev_device_get_sysattr_value(dev: udev_device, attr: c_str) -> c_str;
    pub fn udev_device_set_sysattr_value(dev: udev_device, attr: c_str, value: c_str ) -> c_int;
    pub fn udev_device_get_devpath(dev: udev_device) -> c_str;
    pub fn udev_device_get_subsystem(dev: udev_device) -> c_str;
    pub fn udev_device_get_devtype(dev: udev_device) -> c_str;
    pub fn udev_device_get_syspath(dev: udev_device) -> c_str;
    pub fn udev_device_get_sysname(dev: udev_device) -> c_str;
    pub fn udev_device_get_sysnum(dev: udev_device) -> c_str;
    pub fn udev_device_get_devnode(dev: udev_device) -> c_str;
    pub fn udev_device_get_devnum(dev: udev_device) -> dev_t;
    pub fn udev_device_get_driver(dev: udev_device) -> c_str;
    pub fn udev_device_get_is_initialized(dev: udev_device) -> c_int;
    pub fn udev_device_has_tag(dev: udev_device, tag: c_str) -> c_int;
    pub fn udev_device_get_action(dev: udev_device) -> c_str;
    pub fn udev_device_get_seqnum(dev: udev_device) -> u64;
    pub fn udev_device_get_usec_since_initialized(dev: udev_device) -> u64;

    // Device List
    pub fn udev_device_get_devlinks_list_entry(dev: udev_device) -> udev_list_entry;
    pub fn udev_device_get_tags_list_entry(dev: udev_device) -> udev_list_entry;
    pub fn udev_device_get_properties_list_entry(dev: udev_device) -> udev_list_entry;
    pub fn udev_device_get_sysattr_list_entry(dev: udev_device) -> udev_list_entry;

    // List
    pub fn udev_list_entry_get_name(dev: udev_list_entry) -> c_str;
    pub fn udev_list_entry_get_value(dev: udev_list_entry) -> c_str;
    pub fn udev_list_entry_get_next(dev: udev_list_entry) -> udev_list_entry;

    // Enumerate
    pub fn udev_enumerate_new(udev: udev) -> udev_enumerate;
    pub fn udev_enumerate_unref(e: udev_enumerate) -> udev_enumerate;

    pub fn udev_enumerate_scan_subsystems(e: udev_enumerate) -> c_int;
    pub fn udev_enumerate_scan_devices(e: udev_enumerate) -> c_int;

    pub fn udev_enumerate_add_match_subsystem(e: udev_enumerate, subsystem: c_str) -> c_int;
    pub fn udev_enumerate_add_nomatch_subsystem(e: udev_enumerate, subsystem: c_str) -> c_int;
    pub fn udev_enumerate_add_match_sysattr(e: udev_enumerate, attr: c_str, value: c_str) -> c_int;
    pub fn udev_enumerate_add_nomatch_sysattr(e: udev_enumerate, attr: c_str, value: c_str) -> c_int;
    pub fn udev_enumerate_add_match_property(e: udev_enumerate, attr: c_str, value: c_str) -> c_int;
    pub fn udev_enumerate_add_match_tag(e: udev_enumerate, tag: c_str) -> c_int;
    pub fn udev_enumerate_add_match_is_initialized(e: udev_enumerate) -> c_int;
    pub fn udev_enumerate_add_match_sysname(e: udev_enumerate, tag: c_str) -> c_int;
    pub fn udev_enumerate_add_syspath(e: udev_enumerate, tag: c_str) -> c_int;

    pub fn udev_enumerate_add_match_parent(e: udev_enumerate, dev: udev_device) -> c_int;

    // Enumerate List
    pub fn udev_enumerate_get_list_entry(e: udev_enumerate) -> udev_list_entry;

    // Monitor
    pub fn udev_monitor_new_from_netlink(u: udev, n: c_str) -> udev_monitor;
    pub fn udev_monitor_get_fd(m: udev_monitor) -> c_int;
    pub fn udev_monitor_filter_add_match_subsystem_devtype(m: udev_monitor, s: c_str, d: c_str) -> c_int;
    pub fn udev_monitor_filter_add_match_tag(m: udev_monitor, t: c_str) -> c_int;
    pub fn udev_monitor_filter_remove(m: udev_monitor) -> c_int;
    pub fn udev_monitor_receive_device(m: udev_monitor) -> udev_device;
    pub fn udev_monitor_unref(m: udev_monitor) -> udev_monitor;
    pub fn udev_monitor_enable_receiving(m: udev_monitor) -> c_int;

    pub fn udev_hwdb_new(u: udev) -> udev_list_entry;
    pub fn udev_hwdb_get_properties_list_entry(h: udev_hwdb, m: c_str) -> udev_list_entry;
    pub fn udev_hwdb_unref(h: udev_hwdb) -> udev_hwdb;
}
