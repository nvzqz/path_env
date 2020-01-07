/// Declares Unix-specific items.
macro_rules! cfg_unix {
    (
        $(
            $(#[$meta:meta])*
            $vis:vis fn $f:ident ( $($arg:tt)* ) $(-> $ret:ty)?
            $body:block
        )*
    ) => {
        $(
            #[cfg(any(unix, feature = "_doc-cfg"))]
            #[cfg_attr(feature = "_doc-cfg", doc(cfg(unix)))]
            $(#[$meta])*
            $vis fn $f ( $($arg)* ) $(-> $ret)?
            $body
        )*
    };
    ($($(#[$meta:meta])* $item:item)+) => {
        $(
            $(#[$meta])*
            #[cfg(any(unix, feature = "_doc-cfg"))]
            #[cfg_attr(feature = "_doc-cfg", doc(cfg(unix)))]
            $item
        )+
    };
}

/// Declares Windows-specific items.
macro_rules! cfg_windows {
    (
        $(
            $(#[$meta:meta])*
            $vis:vis fn $f:ident ( $($arg:tt)* ) $(-> $ret:ty)?
            $body:block
        )*
    ) => {
        $(
            #[cfg(any(windows, feature = "_doc-cfg"))]
            #[cfg_attr(feature = "_doc-cfg", doc(cfg(windows)))]
            $(#[$meta])*
            $vis fn $f ( $($arg)* ) $(-> $ret)?
            $body
        )*
    };
    ($($(#[$meta:meta])* $item:item)+) => {
        $(
            $(#[$meta])*
            #[cfg(any(windows, feature = "_doc-cfg"))]
            #[cfg_attr(feature = "_doc-cfg", doc(cfg(windows)))]
            $item
        )+
    };
}
