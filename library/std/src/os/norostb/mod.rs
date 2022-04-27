#![unstable(feature = "norostb", issue = "none")]

pub mod ffi;
pub mod io;

pub mod prelude {
    use super::*;
    pub use ffi::OsStrExt;
    pub use io::{AsHandle, FromHandle, Handle, IntoHandle};
}
