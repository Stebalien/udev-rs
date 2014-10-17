use libudev_c;
use util;

pub struct UdevIterator<'p, T: 'p> {
    #[allow(unused)]
    parent: &'p T,
    entry: libudev_c::udev_list_entry
}

impl<'p, T> UdevIterator<'p, T> {
    pub fn new(parent: &'p T, entry: libudev_c::udev_list_entry) -> UdevIterator<'p, T> {
        UdevIterator {
            parent: parent, 
            entry: entry
        }
    }
}

impl<'p, T> Iterator<(&'p T, &'p str, Option<&'p str>)> for UdevIterator<'p, T> {
    fn next(&mut self) -> Option<(&'p T, &'p str, Option<&'p str>)> {
        if self.entry.is_null() {
            None
        } else {
            let ret = Some((self.parent, unsafe {
                util::c_to_str(libudev_c::udev_list_entry_get_name(self.entry)).unwrap()
            }, unsafe {
                util::c_to_str(libudev_c::udev_list_entry_get_value(self.entry))
            }));
            self.entry = unsafe { libudev_c::udev_list_entry_get_next(self.entry) };
            ret
        }
    }
}
