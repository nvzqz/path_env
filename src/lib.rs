//! Operations on the
//! [`PATH` environment variable](https://en.wikipedia.org/wiki/PATH_(variable)),
//! brought to you by [@NikolaiVazquez]!
//!
//! # Supported Platforms
//!
//! This crate only works for:
//!
//! - Unix-like systems
//! - Windows
//! - Redox
//!
//! It is recommended to use it behind the following `cfg`:
//!
//! ```
//! #[cfg(any(unix, windows, target_os = "redox"))]
//! # mod a {}
//! ```
//!
//! [@NikolaiVazquez]: https://twitter.com/NikolaiVazquez

// This `cfg` allows for the crate to compile on unsupported targets. However,
// it won't be usable.
#![cfg(any(unix, windows, target_os = "redox"))]
#![cfg_attr(feature = "_doc-cfg", feature(doc_cfg))]
#![deny(missing_docs)]

use std::{
    collections::VecDeque,
    env,
    ffi::{OsStr, OsString},
    fmt,
    iter::FromIterator,
    path::Path,
    slice,
};

mod cmp;
mod split;
mod sys;
mod util;

#[macro_use]
mod macros;

pub mod os;
pub mod separator;

use sys::byte_repr::{ByteBufRepr, ByteRepr};

#[doc(inline)]
pub use split::{split, PathEnvSplit};

/// Creates a [`PathEnv`] from the current `PATH` environment variable.
///
/// This is an alias to [`PathEnv::from_var`].
///
/// [`PathEnv`]:           struct.PathEnv.html
/// [`PathEnv::from_var`]: struct.PathEnv.html#method.from_var
#[inline]
pub fn var() -> Option<PathEnv> {
    PathEnv::from_var()
}

/// Removes redundant separators from a `PATH` environment variable.
///
/// # Examples
///
/// If there's nothing but separators, the returned string is empty:
///
/// ```
/// let sep = path_env::separator::STR;
///
/// let rep = format!("{sep}{sep}{sep}{sep}", sep = sep);
/// assert_eq!(path_env::normalize(rep), "");
/// ```
///
/// If there's repeated separators or they appear at the front/back, they'll be
/// removed:
///
/// ```
/// #  let sep = path_env::separator::STR;
/// let bin_a = "/path/to/a/bin";
/// let bin_b = "/path/to/b/bin";
///
/// let norm = format!("{}{}{}", bin_a, sep, bin_b);
/// let path = format!(
///     "{sep}{a}{sep}{sep}{b}{sep}",
///     a = bin_a,
///     b = bin_b,
///     sep = sep
/// );
///
/// assert_eq!(path_env::normalize(path), norm.as_str());
/// ```
///
/// [`OsString`]: https://doc.rust-lang.org/std/ffi/struct.OsString.html
#[inline]
pub fn normalize<P: AsRef<OsStr>>(path: P) -> OsString {
    fn normalize(path: &OsStr) -> OsString {
        let mut result = OsString::with_capacity(path.len());
        let mut parts = split(path);

        if let Some(part) = parts.next() {
            result.push(part);
            for part in parts {
                result.push(separator::OS_STR);
                result.push(part);
            }
        }

        result
    }
    normalize(path.as_ref())
}

/// An owned, mutable `PATH` environment variable.
///
/// # Examples
///
/// This is suitable for prioritizing a certain `bin` path in the `PATH` of a
/// child process:
///
/// ```
/// # fn example() -> Option<()> {
/// use std::process::Command;
///
/// let mut path = path_env::var()?;
/// path.push_front("/path/to/bin");
///
/// Command::new("/path/to/script.sh")
///     .env("PATH", &path)
///     .spawn()
///     .ok()?;
/// # Some(())
/// # }
/// ```
pub struct PathEnv {
    /// The `PATH` variable.
    path: OsString,
    /// A pre-separated list of the components of `path`.
    ///
    /// SAFETY: This is only allowed to reference `path`, never external data.
    /// This may never be exposed as 'static in public API.
    parts: VecDeque<*const Path>,
}

impl Clone for PathEnv {
    fn clone(&self) -> Self {
        let path = self.path.clone();
        let mut parts = self.parts.clone();

        let old_ptr = self.path.as_ptr();
        let new_ptr = path.as_ptr();
        let diff = (new_ptr as isize).wrapping_sub(old_ptr as isize);

        unsafe { Self::offset_parts(&mut parts, diff) };
        Self { path, parts }
    }
}

unsafe impl Send for PathEnv {}
unsafe impl Sync for PathEnv {}

impl From<OsString> for PathEnv {
    fn from(path: OsString) -> Self {
        let parts = split(&path).map(|part| part as *const Path).collect();
        Self { path, parts }
    }
}

impl From<PathEnv> for OsString {
    #[inline]
    fn from(path: PathEnv) -> Self {
        path.into_os_string()
    }
}

impl From<String> for PathEnv {
    #[inline]
    fn from(path: String) -> Self {
        OsString::from(path).into()
    }
}

impl AsRef<OsStr> for PathEnv {
    #[inline]
    fn as_ref(&self) -> &OsStr {
        &self.path
    }
}

impl fmt::Debug for PathEnv {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("PathEnv").field(&self._parts()).finish()
    }
}

impl<P: AsRef<Path>> FromIterator<P> for PathEnv {
    fn from_iter<I: IntoIterator<Item = P>>(iter: I) -> Self {
        let mut iter = iter.into_iter();

        if let Some(part) = iter.next() {
            let mut path = part.as_ref().as_os_str().to_owned();

            let size_hint = match iter.size_hint() {
                (_, Some(upper)) => upper,
                (lower, _) => lower,
            };
            util::reserve_heuristic(&mut path, size_hint);

            iter.for_each(|part| {
                path.push(separator::OS_STR);
                path.push(part.as_ref());
            });

            path.into()
        } else {
            Self::empty()
        }
    }
}

impl<P: AsRef<Path>> Extend<P> for PathEnv {
    fn extend<I: IntoIterator<Item = P>>(&mut self, part_iter: I) {
        let mut part_iter = part_iter.into_iter();
        let part = match part_iter.next() {
            Some(part) => part,
            None => return,
        };

        // Get the old byte buffer start to determine if a reallocation happened
        // and conditionally handle reassignment in `parts` differently.
        let old_buf = self.path.as_bytes();
        let old_ptr = old_buf.as_ptr();
        let old_len = old_buf.len();

        self.path.push(part.as_ref());

        let size_hint = match part_iter.size_hint() {
            (_, Some(upper)) => upper,
            (lower, _) => lower,
        };
        util::reserve_heuristic(&mut self.path, size_hint);

        part_iter.for_each(|part| {
            self.path.push(separator::OS_STR);
            self.path.push(part.as_ref());
        });

        unsafe { self.reconstruct_back(old_ptr, old_len) };
    }
}

impl PathEnv {
    /// Constructs a new, empty `PathEnv`.
    #[inline]
    pub fn empty() -> Self {
        Self {
            path: OsString::new(),
            parts: VecDeque::new(),
        }
    }

    /// Creates an instance by fetching the current `PATH` environment variable.
    #[inline]
    pub fn from_var() -> Option<Self> {
        env::var_os("PATH").map(Self::from)
    }

    cfg_unix! {
        /// Returns the `PATH` returned by running `getconf PATH`.
        ///
        /// This is usually used to get the default `PATH` on POSIX-compliant
        /// systems.
        ///
        /// Use [`os::unix::getconf`] to get the [`OsString`] without parsing.
        ///
        /// [`os::unix::getconf`]: os/unix/fn.getconf.html
        /// [`OsString`]: https://doc.rust-lang.org/std/ffi/struct.OsString.html
        pub fn from_getconf() -> std::io::Result<Self> {
            os::unix::getconf().map(|path| path.into())
        }
    }

    /// Sets the `PATH` environment variable to `self` for the currently running
    /// process.
    ///
    /// See [`env::set_var`] for more info.
    ///
    /// [`env::set_var`]: https://doc.rust-lang.org/std/env/fn.set_var.html
    #[inline]
    pub fn set_var(&self) {
        env::set_var("PATH", &self.path);
    }

    /// Returns the `OsStr` representation of `self`.
    #[inline]
    pub fn as_os_str(&self) -> &OsStr {
        self.path.as_os_str()
    }

    /// Returns the `OsString` representation of `self`.
    #[inline]
    pub fn into_os_string(self) -> OsString {
        self.path
    }

    /// Returns an iterator over the separated paths of `self`.
    // TODO: Make this its own type.
    #[inline]
    pub fn iter(
        &self,
    ) -> impl ExactSizeIterator<Item = &Path> + DoubleEndedIterator<Item = &Path>
    {
        self._parts().iter().map(|part| &**part)
    }

    #[inline]
    fn _parts(&self) -> &VecDeque<&Path> {
        // SAFETY: The pointers in `self.parts` all reference `self.path`.
        unsafe { &*(&self.parts as *const _ as *const _) }
    }

    /// Retrieves the `Path` at `index` in `self`.
    #[inline]
    pub fn get(&self, index: usize) -> Option<&Path> {
        match self._parts().get(index) {
            Some(&path) => Some(path),
            None => None,
        }
    }

    /// Retrieves the `Path` at the start of `self`.
    ///
    /// # Examples
    // TODO: Add example with `push_front`.
    ///
    /// ```
    /// let mut path = path_env::PathEnv::empty();
    /// assert_eq!(path.front(), None);
    ///
    /// path.push_back("/path/to/bin");
    /// assert_eq!(path.front(), Some("/path/to/bin".as_ref()));
    /// ```
    #[inline]
    pub fn front(&self) -> Option<&Path> {
        self.get(0)
    }

    /// Retrieves the `Path` at the end of `self`.
    #[inline]
    pub fn back(&self) -> Option<&Path> {
        self.get(self.len().wrapping_sub(1))
    }

    /// Returns the number of paths in `self`.
    #[inline]
    pub fn len(&self) -> usize {
        self.parts.len()
    }

    /// Returns `true` if `self` contains no paths.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.parts.is_empty()
    }

    /// Removes all contents of `self`.
    #[inline]
    pub fn clear(&mut self) {
        self.parts.clear();
        self.path.clear();
    }

    /// Returns whether `self` contains `path`.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut path_env = path_env::PathEnv::empty();
    ///
    /// let bin_path = "/path/to/my/bin";
    /// path_env.push_back(bin_path);
    ///
    /// assert!(path_env.contains(bin_path));
    /// ```
    #[inline]
    pub fn contains<P: AsRef<Path>>(&self, path: P) -> bool {
        self._parts().contains(&path.as_ref())
    }

    /// Prepends `path` to the front of `self`.
    #[inline]
    pub fn push_front<P: AsRef<Path>>(&mut self, path: P) {
        self._push_front(path.as_ref().as_os_str())
    }

    fn _push_front(&mut self, _part: &OsStr) {
        unimplemented!("TODO: Implement `push_front`")
    }

    /// Appends `path` to the back of `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut path = path_env::PathEnv::empty();
    /// let sep = path_env::separator::STR;
    ///
    /// let bin_a = "/path/to/a/bin";
    /// let bin_b = "/path/to/b/bin";
    /// let new_path = format!("{}{}{}", bin_a, sep, bin_b);
    ///
    /// path.push_back(bin_a);
    /// assert_eq!(path.as_os_str(), bin_a);
    ///
    /// assert_eq!(path.front(), path.back());
    ///
    /// path.push_back(bin_b);
    /// assert_eq!(path.as_os_str(), new_path.as_str());
    ///
    /// assert_eq!(path.front(), Some(bin_a.as_ref()));
    /// assert_eq!(path.back(),  Some(bin_b.as_ref()));
    ///
    /// assert_eq!(path.len(), 2);
    /// ```
    #[inline]
    pub fn push_back<P: AsRef<Path>>(&mut self, path: P) {
        let path = path.as_ref();
        if path.is_empty() {
            return;
        }

        // Get the old byte buffer start to determine if a reallocation happened
        // and conditionally handle reassignment in `parts` differently.
        let old_buf = self.path.as_bytes();
        let old_ptr = old_buf.as_ptr();
        let old_len = old_buf.len();

        if !self.is_empty() {
            self.path.push(separator::OS_STR);
        }
        self.path.push(path);

        unsafe { self.reconstruct_back(old_ptr, old_len) }
    }

    unsafe fn offset_parts(parts: &mut VecDeque<*const Path>, diff: isize) {
        unsafe fn offset(old: *const Path, diff: isize) -> *const Path {
            // Get length by dereferencing to a slice of a zero-sized type
            // first, in order to prevent potential undefined behavior. It's
            // unsure whether this actually helps.
            let len = (*(old as *const [()])).len();

            let new = (old as *const u8).offset(diff);

            slice::from_raw_parts(new, len) as *const [u8] as *const Path
        }

        parts.iter_mut().for_each(|part| {
            *part = offset(*part, diff);
        });
    }

    /// Reconstructs `self.parts` with different strategies depending on whether
    /// `self.path` is a new allocation.
    ///
    /// If `old_ptr` is equal to the new buffer start, then no reallocation
    /// occurred and all pointers in `self.parts` are still correct. Otherwise,
    /// they must be offset by the difference between `old_ptr` and the new
    /// allocation start.
    ///
    /// # Safety
    ///
    /// This should *only* ever be called by functions that append to `self`.
    unsafe fn reconstruct_back(&mut self, old_ptr: *const u8, old_len: usize) {
        let new_buf = self.path.as_bytes();
        let new_ptr = new_buf.as_ptr();
        let ext_buf = OsStr::from_bytes(new_buf.get_unchecked(old_len..));

        // Handle reallocation
        if new_ptr != old_ptr {
            let diff = (new_ptr as isize).wrapping_sub(old_ptr as isize);
            Self::offset_parts(&mut self.parts, diff);
        }

        self.parts
            .extend(split(ext_buf).map(|part| part as *const Path));
    }

    /// Removes the first path in `self`.
    pub fn pop_front(&mut self) {
        self.pop_front_n(1);
    }

    /// Removes (at most) the first `n` paths in `self`.
    ///
    /// It is fine for `n` to be greater than the number of paths in `self`.
    pub fn pop_front_n(&mut self, _n: usize) {
        unimplemented!("TODO: Implement `pop_front_n`")
    }

    /// Removes the last path in `self`.
    #[inline]
    pub fn pop_back(&mut self) {
        self.pop_back_n(1);
    }

    /// Removes (at most) the last `n` paths in `self`.
    ///
    /// It is fine for `n` to be greater than the number of paths in `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn example() -> Option<()> {
    /// let mut path = path_env::var()?;
    /// let original = path.clone();
    ///
    /// let bin_a = "/path/to/a/bin";
    /// let bin_b = "/path/to/a/bin";
    ///
    /// path.push_back(bin_a);
    /// path.push_back(bin_b);
    ///
    /// path.pop_back_n(2);
    ///
    /// assert_eq!(path, original);
    /// # Some(())
    /// # }
    /// # example().unwrap();
    /// ```
    pub fn pop_back_n(&mut self, n: usize) {
        let (i, end) = match self.len().saturating_sub(n) {
            0 => (0, self.path.as_ptr()),
            i => match self.get(i - 1) {
                Some(part) => {
                    let part = part.as_bytes();

                    // SAFETY: `part` references valid data in `self.path`
                    // and is thus before the end of `self.data`.
                    let end = unsafe { part.as_ptr().add(part.len()) };

                    (i, end)
                }
                None => return,
            },
        };
        unsafe { self.path.set_end(end) };
        self.parts.truncate(i);
    }
}
