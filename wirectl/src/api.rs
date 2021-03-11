use std::fmt::Debug;

/// Used by [`crate::interface::WgInterface`] to determine which underlaying API to use
#[derive(Debug)]
pub enum WgApi {
    IPC,
    #[cfg(target_os = "linux")]
    Linux,
    #[cfg(any(target_os = "openbsd", target_os = "freebsd"))]
    BSD,
}


