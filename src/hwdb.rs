use std::iter;

use libudev_c;

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

pub fn hwdb<'u>(udev: &'u Udev, hwdb: libudev_c::udev_hwdb) -> Result<Hwdb<'u>, ()> {
    // TODO: check errno and panic on oom?
    if hwdb.is_null() {
        Err(())
    } else {
        Ok(Hwdb {
            udev: udev,
            hwdb: hwdb
        })
    }
}

impl<'u> Hwdb<'u> {
    pub fn query<'s>(&'s mut self, modalias: &str) -> HwdbQuery<'s, 'u> {
        // HACK: take reference here because we can't reference self.hwdb inside the closure.
        let db = self.hwdb;
        HwdbQuery {
            hwdb: self,
            entry: modalias.with_c_str(|modalias| {
                unsafe { libudev_c::udev_hwdb_get_properties_list_entry(db, modalias) }
            })
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
