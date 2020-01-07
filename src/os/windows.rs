//! Windows-specific definitions.

/// The default `PATH` value.
pub const DEFAULT_PATH: &str = r"%SystemRoot%\system32;%SystemRoot%;%SystemRoot%\System32\Wbem";

/// The default `PATH` value extended with `%SYSTEMROOT%\System32\WindowsPowerShell\v1.0\`.
pub const DEFAULT_PATH_EXT: &str = r"%SystemRoot%\system32;%SystemRoot%;%SystemRoot%\System32\Wbem;%SYSTEMROOT%\System32\WindowsPowerShell\v1.0\";
