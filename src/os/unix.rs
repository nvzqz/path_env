//! Unix-specific definitions.

use std::{ffi::OsString, io, os::unix::ffi::OsStringExt, process::Command};

/// Returns the `PATH` returned by running `getconf PATH`.
///
/// This is usually used to get the default `PATH` on POSIX-compliant systems.
pub fn getconf() -> io::Result<OsString> {
    Command::new("getconf")
        .arg("PATH")
        .output()
        .map(|output| OsString::from_vec(output.stdout))
}
