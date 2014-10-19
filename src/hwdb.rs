use std::iter;
use libc::EINVAL;

use libudev_c;
use udev;
use util;

use udev::Udev;
use iterator::UdevIterator;

pub struct Hwdb<'u> {
    pub udev: &'u Udev,
    hwdb: libudev_c::udev_hwdb
}

pub struct HwdbQuery<'h, 'u: 'h> {
    pub hwdb: &'h mut Hwdb<'u>,
    entry: libudev_c::udev_list_entry
}

// Crate Private
pub fn hwdb(udev: &Udev) -> Result<Hwdb, i32> {
    match util::check_errno(|| unsafe {
        libudev_c::udev_hwdb_new(udev::get_udev_ctx(udev))
    }) {
        Ok(Some(hwdb))  => Ok(Hwdb { udev: udev, hwdb: hwdb }),
        Ok(None)        => Err(0i32),
        Err(EINVAL)     => fail!("BUG"),
        Err(e)          => Err(e)
    }
}

impl<'u> Hwdb<'u> {
    pub fn query<'s>(&'s mut self, modalias: &str) -> HwdbQuery<'s, 'u> {
        // HACK: take reference here because we can't reference self.hwdb inside the closure.
        let entry = modalias.with_c_str(|modalias| {
            unsafe { libudev_c::udev_hwdb_get_properties_list_entry(self.hwdb, modalias) }
        });

        HwdbQuery {
            hwdb: self,
            entry: entry
        }
    }
}

impl<'u> HwdbQuery<'u, 'u> {
    pub fn iter(&self) -> iter::Map<(&Hwdb, &str, Option<&str>),(&str, &str),UdevIterator<Hwdb>> {
        UdevIterator::new(self.hwdb, self.entry).map(|(_, k, v)| (k, v.unwrap()))
    }
}


#[unsafe_destructor]
impl<'u> Drop for Hwdb<'u> {
    fn drop(&mut self) {
        unsafe { libudev_c::udev_hwdb_unref(self.hwdb) };
    }
}
