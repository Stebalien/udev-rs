use std::iter;

use libudev_c;
use iterator;

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
pub unsafe fn hwdb(udev: &Udev, hwdb: libudev_c::udev_hwdb) -> Hwdb {
    Hwdb { udev: udev, hwdb: hwdb }
}

impl<'u> Hwdb<'u> {
    /// Query the hardware database.
    ///
    /// # Note
    ///
    /// Only one query can exist at a time.
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

impl<'h, 'u> HwdbQuery<'h, 'u> {
    /// Iterate over the properties returned by this query.
    pub fn iter(&self) -> iter::Map<(&Hwdb, &str, Option<&str>),(&str, &str),UdevIterator<Hwdb>> {
        unsafe { iterator::udev_iterator(self.hwdb, self.entry) }.map(|(_, k, v)| (k, v.unwrap()))
    }
}


#[unsafe_destructor]
impl<'u> Drop for Hwdb<'u> {
    fn drop(&mut self) {
        unsafe { libudev_c::udev_hwdb_unref(self.hwdb) };
    }
}
