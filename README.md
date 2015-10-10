Udev bindings for rust
======================

*NOTICE: This library is currently unmaintained. If you would like to take over,
please file an issue. libudev is poorly designed (and in some places, plain
broken) so I have given up trying to tame it into a sane rust library.*

These bindings effectivly map one-to-one to libudev (but calling them is much
simpler/nicer than C).

[API](https://stebalien.github.io/udev-rs/udev/)

## !!!WARNING!!!

This is alpha quality and my first rust project. Please comment/review my code
(I'm probably doing quite a few things wrong).

## Implementation Notes

### Threading

One caviat to note is that this library is not thread safe (you can't share
data structures between threads). Unfortunately, this means a udev context and
all udev objects created from it will have to stay within the same thread.

### Monitors

I currently don't expose the underlying monitor file descriptor and don't
provide a way to asynchronously wait on a monitor. While I would like this
feature, I can't see a safe way to provide it at the moment.

### Enumerators

The enumerators API is a little funky because it matches the underlying libudev
API as much as possible. Specifically, you have to remember to call
`scan_devices`, `scan_subsystems`, or at least `add_device` before iterating to
actually do anything useful. At first, I included an implicit device scan in
the iter function but this isn't quite as powerful. As is, you iterativly build
up a list of devices in an enumerator (by repeatedly calling `match_*` and then
`scan_*`).

In the future, I might consider adding a simpler (saner) query interface.

### Iterator Proliferation

You might notice the plethera of iterators that basically just map/filter the
private UdevIterator. I used to use map/filter but then I ended up exposing
implementation details in return types. If there is a better way to do this
(that rust currently supports...), please tell me.

## Examples

### List TTY device nodes

```rust
let udev = Udev::new();
for dev in udev.enumerator().match_subsystem("tty").scan_devices().iter() {
    assert!(dev.subsystem().unwrap() == "tty");
    if dev.sysname().starts_with("tty") {
        match dev.devnode() {
            Some(devnode) => println!("{}", devnode.display()),
            None => ()
        }
    }
}
```

### Monitor for added/removed block devices
```rust
let udev = Udev::new();
for (e, d) in udev.monitor().unwrap().filter_by_subsystem("block").iter() {
    match e.action {
        AddAction | RemoveAction => println!("{} {}", e.action, d),
        _ => ()
    };
}
```
