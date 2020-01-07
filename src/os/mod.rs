//! OS-specific functionality.

cfg_unix! {
    pub mod unix;
}

cfg_windows! {
    pub mod windows;
}
