#![deny(unsafe_op_in_unsafe_fn)]

pub mod alloc;
pub mod args;
#[path = "../unix/cmath.rs"]
pub mod cmath;
pub mod env;
pub mod fs;
pub mod io;
pub mod locks;
pub mod net;
pub mod os;
#[path = "../unix/os_str.rs"]
pub mod os_str;
#[path = "../unix/path.rs"]
pub mod path;
pub mod pipe;
pub mod process;
pub mod stdio;
pub mod thread;
#[cfg(target_thread_local)]
pub mod thread_local_dtor;
pub mod thread_local_key;
pub mod time;

mod common;
pub use common::*;

// This function is needed by the panic runtime. The symbol is named in
// pre-link args for the target specification, so keep that in sync.
#[cfg(not(test))]
#[no_mangle]
// NB. used by both libunwind and libpanic_abort
pub extern "C" fn __rust_abort() {
    abort_internal();
}

/// # Safety
///
/// Must be called only once during runtime initialization.
///
/// # Note
///
/// This is not guaranteed to run, for example when Rust code is called externally.
pub unsafe fn init(_: isize, _: *const *const u8) {
    // FIXME this is not guaranteed to run.
    unsafe {
        stdio::init();
    }
}

/// # Safety
///
/// Must be called only once during runtime cleanup.
///
/// # Note
///
/// This is not guaranteed to run, for example when Rust code is called externally.
pub unsafe fn cleanup() {}

fn cvt_err(err: norostb_rt::Error) -> crate::io::Error {
    use crate::io::{const_io_error, ErrorKind};
    use norostb_rt::Error;
    match err {
        Error::Unknown => const_io_error!(ErrorKind::Uncategorized, "uncategorized error"),
        Error::InvalidOperation => const_io_error!(ErrorKind::Unsupported, "invalid operation"),
        Error::DoesNotExist => const_io_error!(ErrorKind::NotFound, "does not exist"),
        Error::AlreadyExists => const_io_error!(ErrorKind::AlreadyExists, "already exists"),
        Error::Cancelled => const_io_error!(ErrorKind::Uncategorized, "cancelled"),
        Error::CantCreateObject => const_io_error!(ErrorKind::InvalidInput, "can't create object"),
        Error::InvalidObject => const_io_error!(ErrorKind::InvalidInput, "invalid object"),
    }
}

const ERR_UNSET: crate::io::Error =
    crate::io::const_io_error!(crate::io::ErrorKind::Uncategorized, "handle is not set");
