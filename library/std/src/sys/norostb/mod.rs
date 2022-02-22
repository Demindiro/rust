#![deny(unsafe_op_in_unsafe_fn)]

pub mod alloc;
pub mod args;
#[path = "../unix/cmath.rs"]
pub mod cmath;
pub mod condvar;
pub mod env;
pub mod fs;
pub mod io;
pub mod mutex;
pub mod net;
pub mod os;
#[path = "../unix/os_str.rs"]
pub mod os_str;
#[path = "../unix/path.rs"]
pub mod path;
pub mod pipe;
pub mod process;
pub mod rwlock;
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
