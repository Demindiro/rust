#![unstable(feature = "norostb", issue = "none")]

pub mod ffi;
pub mod io;

pub use crate::sys::io::{
    create, finish_job, open, poll, query, query_next, read, seek, take_job, write,
};
pub use norostb_rt::kernel::io::{Job, ObjectInfo};

pub mod prelude {
    use super::*;
    pub use ffi::OsStrExt;
    pub use io::{AsHandle, FromHandle, Handle, IntoHandle};
}
