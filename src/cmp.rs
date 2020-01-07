use super::PathEnv;
use std::{
    cmp,
    ffi::{OsStr, OsString},
};

impl PartialEq for PathEnv {
    fn eq(&self, other: &Self) -> bool {
        // We compare `parts` instead of the `OsStr` representation because it
        // uses the correct comparison semantics.
        self._parts() == other._parts()
    }
}

impl Eq for PathEnv {}

impl PartialEq<OsStr> for PathEnv {
    #[inline]
    fn eq(&self, path: &OsStr) -> bool {
        self.partial_cmp(path) == Some(cmp::Ordering::Equal)
    }
}

impl PartialEq<OsString> for PathEnv {
    #[inline]
    fn eq(&self, path: &OsString) -> bool {
        self == path.as_os_str()
    }
}

impl PartialEq<str> for PathEnv {
    #[inline]
    fn eq(&self, path: &str) -> bool {
        self == OsStr::new(path)
    }
}

impl PartialEq<String> for PathEnv {
    #[inline]
    fn eq(&self, path: &String) -> bool {
        self == path.as_str()
    }
}

impl PartialEq<PathEnv> for OsStr {
    #[inline]
    fn eq(&self, path: &PathEnv) -> bool {
        path == self
    }
}

impl PartialEq<PathEnv> for OsString {
    #[inline]
    fn eq(&self, path: &PathEnv) -> bool {
        path == self
    }
}

impl PartialEq<PathEnv> for str {
    #[inline]
    fn eq(&self, path: &PathEnv) -> bool {
        path == self
    }
}

impl PartialEq<PathEnv> for String {
    #[inline]
    fn eq(&self, path: &PathEnv) -> bool {
        path == self
    }
}

impl PartialOrd for PathEnv {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PathEnv {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        // We compare `parts` instead of the `OsStr` representation because it
        // uses the correct comparison semantics.
        self._parts().cmp(other._parts())
    }
}

impl PartialOrd<OsStr> for PathEnv {
    fn partial_cmp(&self, path: &OsStr) -> Option<cmp::Ordering> {
        let mut path_iter = crate::split(path);

        for &part in self._parts() {
            if let Some(path) = path_iter.next() {
                match part.cmp(path) {
                    cmp::Ordering::Equal => continue,
                    non_equal => return Some(non_equal),
                }
            } else {
                return Some(cmp::Ordering::Greater);
            }
        }

        if path_iter.next().is_some() {
            Some(cmp::Ordering::Less)
        } else {
            Some(cmp::Ordering::Equal)
        }
    }
}

impl PartialOrd<PathEnv> for OsStr {
    #[inline]
    fn partial_cmp(&self, path: &PathEnv) -> Option<cmp::Ordering> {
        path.partial_cmp(self).map(|ord| ord.reverse())
    }
}

impl PartialOrd<OsString> for PathEnv {
    #[inline]
    fn partial_cmp(&self, path: &OsString) -> Option<cmp::Ordering> {
        self.partial_cmp(path.as_os_str())
    }
}
