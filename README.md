## dm_ioctl

A thin but type-safe wrapper around Linux’s device-mapper ioctls.
Does not use `libdevicemapper`, does not interact with `udev` at all,
and is (not yet, but will be) usable in a no-std (or more precisely
no-C-library) context.

Originally the “core” component of [devicemapper-rs][], now maintained
independently.

The version number of this crate is equal to the newest version of the
device-mapper ioctl API that it understands; this is currently 4.48.0.
That API version was introduced in Linux kernel 6.4.  (Depending on
what you want to do with it, though, the crate should work fine with
much older kernels.)

[devicemapper-rs]: https://github.com/stratis-storage/devicemapper-rs/

### Development status

You probably shouldn’t use this right now unless you’re me.

### Documentation

[API Documentation](https://docs.rs/dm_ioctl).

[Devicemapper documentation (such as it is)](https://www.kernel.org/doc/Documentation/device-mapper/)

### How to contribute

This is a personal project.  Suggestions, bug reports, and/or patches
sent to <zack@owlfolio.org> will be considered, but I am not promising
anything more than that.

#### Changes that will *not* be considered

Please do not send me patches that do any of these things:

- restore any sort of interaction with udev
- introduce features that would get in the way of making the crate
  no-std compatible
- increase the minimum version number of any dependency, including
  Rust the language, without a good reason.  “A newer version is
  available” is not a good reason.  “The minimum version is past its
  end-of-life date” is not a good reason.
- commit `Cargo.lock`

### License

[Mozilla Public License 2.0](https://www.mozilla.org/MPL/2.0/FAQ.html)
