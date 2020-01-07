use std::{ffi::OsStr, iter::FusedIterator, path::Path};

use crate::sys::byte_repr::ByteRepr;

/// Creates an iterator over the paths in a `PATH` environment variable.
///
/// See [`PathEnvSplit`] for more info.
///
/// [`PathEnvSplit`]: struct.PathEnvSplit.html
#[inline]
pub fn split<P: ?Sized + AsRef<OsStr>>(path: &P) -> PathEnvSplit {
    PathEnvSplit::new(path)
}

/// An iterator over the paths in a `PATH` environment variable.
///
/// # Examples
///
/// The `PATH` environment variable must be retrieved first because the iterator
/// does not take ownership.
///
/// ```
/// # fn example() -> Option<()> {
/// let path_var = std::env::var_os("PATH")?;
///
/// for path in path_env::split(&path_var) {
///     println!("{:?}", path);
/// }
/// # Some(())
/// # }
/// ```
///
/// The iterator ignores any empty paths in the provided `path`:
///
/// ```
/// use std::path::Path;
///
/// let bin_path = "/Users/Müller/bin";
///
/// let path_env = format!(
///     "{bin}{sep}{bin}{sep}{sep}{bin}{sep}",
///     bin = bin_path,
///     sep = path_env::separator::STR
/// );
///
/// let mut paths = path_env::split(&path_env);
/// let bin_path = Path::new(bin_path);
///
/// assert_eq!(paths.next(), Some(bin_path));
/// assert_eq!(paths.next(), Some(bin_path));
/// assert_eq!(paths.next(), Some(bin_path));
/// assert_eq!(paths.next(), None);
/// ```
///
/// The iterator also supports reverse iteration:
///
/// ```
/// # use std::path::Path;
/// let sep = path_env::separator::STR;
/// let bin_a = "/path/to/a/bin";
/// let bin_b = "/path/to/b/bin";
/// let path_env = format!("{}{}{}", bin_a, sep, bin_b);
///
/// let paths = path_env::split(&path_env)
///     .rev()
///     .collect::<Vec<_>>();
///
/// assert_eq!(paths[0], Path::new(bin_b));
/// assert_eq!(paths[1], Path::new(bin_a));
/// assert_eq!(paths.len(), 2);
/// ```
#[derive(Clone, Debug)]
pub struct PathEnvSplit<'a>(&'a OsStr);

impl<'a> PathEnvSplit<'a> {
    /// Creates an instance from a reference to the contents of a `PATH`
    /// environment variable.
    #[inline]
    pub fn new<P: ?Sized + AsRef<OsStr>>(path: &'a P) -> Self {
        Self(path.as_ref())
    }
}

impl<'a> Iterator for PathEnvSplit<'a> {
    type Item = &'a Path;

    // TODO: Handle quote pairs on Windows.
    fn next(&mut self) -> Option<&'a Path> {
        fn next_separator(bytes: &[u8]) -> Option<usize> {
            #[cfg(feature = "memchr")]
            {
                memchr::memchr(crate::separator::U8, bytes)
            }

            #[cfg(not(feature = "memchr"))]
            {
                bytes.iter().position(|&b| b == crate::separator::U8)
            }
        }

        let mut path = self.0.as_bytes();

        while let Some(i) = next_separator(path) {
            let next = &path[..i];
            path = &path[(i + 1)..];

            if next.is_empty() {
                continue;
            } else {
                unsafe {
                    self.0 = OsStr::from_bytes(path);
                    return Some(Path::from_bytes(next));
                }
            }
        }

        self.0 = Default::default();

        if path.is_empty() {
            None
        } else {
            Some(unsafe { Path::from_bytes(path) })
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        fn count(bytes: &[u8]) -> usize {
            #[cfg(feature = "bytecount")]
            {
                bytecount::count(bytes, crate::separator::U8)
            }

            #[cfg(not(feature = "bytecount"))]
            #[allow(clippy::naive_bytecount)]
            {
                bytes.iter().filter(|&&b| b == crate::separator::U8).count()
            }
        }
        (0, Some(count(self.0.as_bytes()) + 1))
    }

    #[inline]
    fn last(mut self) -> Option<&'a Path> {
        self.next_back()
    }
}

impl<'a> DoubleEndedIterator for PathEnvSplit<'a> {
    // TODO: Handle quote pairs on Windows.
    fn next_back(&mut self) -> Option<&'a Path> {
        fn next_separator(bytes: &[u8]) -> Option<usize> {
            bytes.iter().rev().position(|&b| b == crate::separator::U8)
        }

        let mut path = self.0.as_bytes();

        while let Some(i) = next_separator(path) {
            let next = &path[(i + 1)..];
            path = &path[..i];

            if next.is_empty() {
                continue;
            } else {
                unsafe {
                    self.0 = OsStr::from_bytes(path);
                    return Some(Path::from_bytes(next));
                }
            }
        }

        self.0 = Default::default();

        if path.is_empty() {
            None
        } else {
            Some(unsafe { Path::from_bytes(path) })
        }
    }
}

impl FusedIterator for PathEnvSplit<'_> {}
