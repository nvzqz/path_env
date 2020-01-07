// The `cfg` is here despite that in `lib.rs` to ensure we don't accidentally
// start using a byte representation for a platform that doesn't use it.
#[cfg(any(unix, windows, target_os = "redox"))]
pub mod byte_repr;
