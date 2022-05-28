use crate::io as std_io;

pub mod memchr {
    pub use core::slice::memchr::{memchr, memrchr};
}

pub fn unsupported<T>() -> std_io::Result<T> {
    Err(unsupported_err())
}

pub fn unsupported_err() -> std_io::Error {
    std_io::const_io_error!(
        std_io::ErrorKind::Unsupported,
        "operation not supported on this platform",
    )
}

pub fn decode_error_kind(_code: i32) -> crate::io::ErrorKind {
    crate::io::ErrorKind::Uncategorized
}

pub fn abort_internal() -> ! {
    super::os::exit(i32::MAX)
}

pub fn hashmap_random_keys() -> (u64, u64) {
    (1, 2)
}
