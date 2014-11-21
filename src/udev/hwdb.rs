// This file is part of udev-rs.
// 
// Copyright 2014 Steven Allen <steven@stebalien.com>
// 
// udev-rs is free software; you can redistribute it and/or modify it
// under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation; either version 2.1 of the License, or
// (at your option) any later version.
// 
// udev-rs is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// Lesser General Public License for more details.
// 
// You should have received a copy of the GNU Lesser General Public License
// along with udev-rs; If not, see <http://www.gnu.org/licenses/>.

use udev::{
    libudev_c,
    iterator
};
use udev::iterator::MappedIterator;
use udev::udev::Udev;

#[doc(hidden)]
pub type HwdbIterator<'p> = MappedIterator<'p, Hwdb<'p>, (&'p str, &'p str)>;

pub struct Hwdb<'u> {
    udev: &'u Udev,
    hwdb: libudev_c::udev_hwdb
}

pub struct Query<'h, 'u: 'h> {
    hwdb: &'h mut Hwdb<'u>,
    entry: libudev_c::udev_list_entry
}

// Crate Private
pub unsafe fn hwdb(udev: &Udev, hwdb: libudev_c::udev_hwdb) -> Hwdb {
    Hwdb { udev: udev, hwdb: hwdb }
}

impl<'u> Hwdb<'u> {
    /// Get the udev context.
    pub fn udev(&self) -> &Udev {
        self.udev
    }

    /// Query the hardware database.
    ///
    /// # Note
    ///
    /// Only one query can exist at a time.
    pub fn query<'s>(&'s mut self, modalias: &str) -> Query<'s, 'u> {
        // HACK: take reference here because we can't reference self.hwdb inside the closure.
        let entry = modalias.with_c_str(|modalias| {
            unsafe { libudev_c::udev_hwdb_get_properties_list_entry(self.hwdb, modalias) }
        });

        Query {
            hwdb: self,
            entry: entry
        }
    }
}

impl<'h, 'u> Query<'h, 'u> {
    /// Iterate over the properties returned by this query.
    pub fn iter(&self) -> HwdbIterator {
        unsafe {
            iterator::iterator(self.hwdb, self.entry)
        }.map(|(_, key, value)| (key, value.unwrap()))
    }
}

#[unsafe_destructor]
impl<'u> Drop for Hwdb<'u> {
    fn drop(&mut self) {
        unsafe { libudev_c::udev_hwdb_unref(self.hwdb) };
    }
}
