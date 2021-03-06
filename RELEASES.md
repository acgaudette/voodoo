Version 0.4.0 (UNRELEASED)
==========================

Breaking Changes
----------------

* https://github.com/cogciprocate/voodoo/pull/18
* https://github.com/cogciprocate/voodoo/pull/11

 
Version 0.3.1 (2018-03-11)
==========================

* Minor documentation updates.


Version 0.3.0 (2018-01-23)
==========================

Breaking Changes
----------------
* `DeviceMemory::map` is now `unsafe`.
* `Buffer::bind_memory` and `Image::bind_memory` are now unsafe.


Version 0.2.1 (2017-10-16)
==========================

* Create a workspace, update readme, and other tweaks.


Version 0.2.0 (2017-10-14)
==========================

Breaking Changes
----------------
* `vks` has been updated to 0.21.0.
* The `Window` type and a few other xlib types have changed.
* `voodoo_winit` has been moved into its own crate.


Version 0.1.2 (2017-10-14)
==========================

New
---
All core and many extension functions have documentation links.

Fixes
-----
* Xlib types are temporarily fixed.


Version 0.1.1 (2017-10-13)
==========================

New
---

* Debug report printing can now be optionally enabled during instance creation
  by passing `true` to the `InstanceBuilder::print_debug_report` method.
  Enabling this automatically loads the necessary extensions and wires up a
  callback function which simply prints the messages to stdout.
* Additional safe API functions have been added.

Breaking Changes
----------------

* The process of creating and accessing device queues has been redesigned. See
  the [`hello.rs`] example for usage.


[`hello.rs`]: https://github.com/cogciprocate/voodoo/blob/master/examples/hello.rs


Version 0.1.0 (2017-10-12)
==========================

Features
--------

* An intuitive and idiomatic interface
* Zero additional overhead
  * No unnecessary allocations
  * No intermediate structs or extra copying
  * Builders compile to direct assignment
* Thread-safe allocation / destruction
  * Safety escape hatches available everywhere
* A minimum of boilerplate
* Non-opinionated and nothing hidden
* Complete API coverage
* Useful documentation

Getting Started
---------------

Ensure that Vulkan drivers are installed for your device. Add the following to
your project's Cargo.toml:

```toml
[dependencies]
voodoo = "0.1"
```

And add the following to your crate root (lib.rs or main.rs):
```rust
extern crate voodoo;
```


Example
-------

Create an instance:

```rust
extern crate voodoo;

use voodoo::{Result as VdResult, Instance, ApplicationInfo, Loader};
use std::ffi::CString;

/// Initializes and returns a new loader and instance with all available
/// extension function pointers loaded.
fn init_instance() -> VdResult<Instance> {
    let app_name = CString::new("Hello!")?;

    let app_info = ApplicationInfo::builder()
        .application_name(&app_name)
        .application_version((1, 0, 0))
        .api_version((1, 0, 0))
        .build();

    let loader = Loader::new()?;

    Instance::builder()
        .application_info(&app_info)
        .enabled_extensions(&loader.enumerate_instance_extension_properties()?)
        .build(loader)
}

fn main() {
    let _instance = init_instance().unwrap();
}

```

See [`hello.rs`] for a complete, working example adapted from
[https://vulkan-tutorial.com/](https://vulkan-tutorial.com/).


Status
------

* API coverage:
  * Core: 100%
  * Extensions: 70%
* Documentation: 5%
* Stability: 90%