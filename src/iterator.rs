use libudev_c;
use util;

// TODO: I could do all of this functionally (map/filter style) but that would make the return
// types a total mess. Therefore, I don't.
//
// When rust finally adds that feature, I can get rid of most of this file...

// Create private.
pub struct UdevIterator<'p, Sized? T: 'p> {
    #[allow(unused)]
    pub parent: &'p T,
    pub entry: libudev_c::udev_list_entry
}

impl<'p, Sized? T> Iterator<(&'p str, Option<&'p str>)> for UdevIterator<'p, T> {
    fn next(&mut self) -> Option<(&'p str, Option<&'p str>)> {
        if self.entry.is_null() {
            None
        } else {
            let ret = Some((unsafe {
                util::c_to_str(libudev_c::udev_list_entry_get_name(self.entry)).unwrap()
            }, unsafe {
                util::c_to_str(libudev_c::udev_list_entry_get_value(self.entry))
            }));
            self.entry = unsafe { libudev_c::udev_list_entry_get_next(self.entry) };
            ret
        }
    }
}

// Generic path iterator.

pub unsafe fn path_iterator<'a>(parent: &'a Sized, entry: libudev_c::udev_list_entry) -> PathIterator<'a> {
    PathIterator {
        iter: key_iterator(parent, entry)
    }
}

pub struct PathIterator<'a> {
    iter: KeyIterator<'a>
}

impl<'a> Iterator<Path> for PathIterator<'a> {
    #[inline]
    fn next(&mut self) -> Option<Path> {
        self.iter.next().map(Path::new)
    }
}

// Generic key iterator

pub unsafe fn key_iterator<'a>(parent: &'a Sized, entry: libudev_c::udev_list_entry) -> KeyIterator<'a> {
    KeyIterator {
        iter: UdevIterator {
            parent: parent,
            entry: entry,
        }
    }
}

pub struct KeyIterator<'a> {
    pub iter: UdevIterator<'a, Sized+'a>
}

impl<'a> Iterator<&'a str> for KeyIterator<'a> {
    #[inline]
    fn next(&mut self) -> Option<&'a str> {
        self.iter.next().map(|(key, _)| key)
    }
}

// Generic key value iterator

pub unsafe fn key_value_iterator(parent: &Sized, entry: libudev_c::udev_list_entry) -> KeyValueIterator {
    KeyValueIterator {
        iter: UdevIterator {
            parent: parent,
            entry: entry,
        }
    }
}

pub struct KeyValueIterator<'a> {
    iter: UdevIterator<'a, Sized+'a>
}

impl<'a> Iterator<(&'a str, &'a str)> for KeyValueIterator<'a> {
    #[inline]
    fn next(&mut self) -> Option<(&'a str, &'a str)> {
        self.iter.next().map(|(key, value)|(key, value.unwrap()))
    }
}


// Generic key optional value iterator.

pub unsafe fn key_opt_value_iterator(parent: &Sized, entry: libudev_c::udev_list_entry) -> KeyOptValueIterator {
    KeyOptValueIterator {
        iter: UdevIterator {
            parent: parent,
            entry: entry,
        }
    }
}

pub struct KeyOptValueIterator<'a> {
    iter: UdevIterator<'a, Sized+'a>
}

impl<'a> Iterator<(&'a str, Option<&'a str>)> for KeyOptValueIterator<'a> {
    #[inline]
    fn next(&mut self) -> Option<(&'a str, Option<&'a str>)> {
        self.iter.next()
    }
}

