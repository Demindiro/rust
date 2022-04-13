#![unstable(feature = "norostb", issue = "none")]

#[path = "../unix/ffi/os_str.rs"]
mod os_str;

#[unstable(feature = "norostb", issue = "none")]
pub use self::os_str::{OsStrExt, OsStringExt};
