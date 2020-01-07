//! Constants for the path deliminator in a `PATH` environment variable.
//!
//! This is `:` on Unix-like systems, and `;` on Windows and Redox.

use std::ffi::OsStr;

#[cfg(all(unix, not(target_os = "redox")))]
const _SEPARATOR: u8 = b':';

// Redox uses semicolon, unlike other targets in the Unix family.
//
// See: https://github.com/rust-lang/rust/blob/0498da9a3dc061f604fcfb9b56bd889e07f2b7e2/src/libstd/sys/unix/os.rs#L28-L34
#[cfg(any(windows, target_os = "redox"))]
const _SEPARATOR: u8 = b';';

/// The `PATH` separator as [`u8`].
///
/// This is `:` on Unix-like systems, and `;` on Windows and Redox.
///
/// [`U8`]: https://doc.rust-lang.org/std/primitive.u8.html
pub const U8: u8 = _SEPARATOR;

/// The `PATH` separator as [`char`].
///
/// This is `:` on Unix-like systems, and `;` on Windows and Redox.
///
/// [`char`]: https://doc.rust-lang.org/std/primitive.char.html
pub const CHAR: char = U8 as char;

/// The `PATH` separator as [`&str`].
///
/// This is `:` on Unix-like systems, and `;` on Windows and Redox.
///
/// [`&str`]: https://doc.rust-lang.org/std/primitive.str.html
pub const STR: &str = {
    // Required since `str::from_utf8_unchecked` does not work in `const`.
    union Cast<'a> {
        utf8: &'a str,
        raw: &'a [u8],
    }
    // SAFETY: The separator byte is a valid UTF-8 encoded scalar by itself.
    unsafe { Cast { raw: &[U8] }.utf8 }
};

/// The `PATH` separator as [`&OsStr`].
///
/// This is `:` on Unix-like systems, and `;` on Windows and Redox.
///
/// [`&OsStr`]: https://doc.rust-lang.org/std/ffi/struct.OsStr.html
pub const OS_STR: &OsStr = {
    // Required since `OsStr::new` does not work in `const`.
    union Cast<'a> {
        utf8: &'a str,
        os: &'a OsStr,
    }
    // SAFETY: This is fine because `str` implements `AsRef<OsStr>`.
    unsafe { Cast { utf8: STR }.os }
};
