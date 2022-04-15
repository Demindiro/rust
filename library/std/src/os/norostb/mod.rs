#![unstable(feature = "norostb", issue = "none")]

pub mod ffi;

pub use crate::sys::io::{create, finish_job, open, query, query_next, read, take_job, write};
pub use norostb_rt::kernel::io::{Job, ObjectInfo};
