use crate::ffi::{OsStr, OsString};
use crate::io;
use crate::os::norostb::ffi::{OsStrExt, OsStringExt};

use norostb_rt::args;

#[derive(Debug)]
pub struct Args(args::Args);

impl Iterator for Args {
    type Item = OsString;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(OsStr::from_bytes).map(Into::into)
    }
}

impl ExactSizeIterator for Args {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl DoubleEndedIterator for Args {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(OsStr::from_bytes).map(Into::into)
    }
}

#[derive(Debug)]
pub struct Env(args::Env);

impl Iterator for Env {
    type Item = (OsString, OsString);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(key, val)| {
            (OsString::from_vec(key.into_owned()), OsString::from_vec(val.into_owned()))
        })
    }
}

pub fn args() -> Args {
    Args(args::Args::new())
}

pub fn env() -> Env {
    Env(args::Env::new())
}

pub fn getenv(key: &OsStr) -> Option<OsString> {
    args::Env::get(OsStr::as_bytes(key)).map(|val| OsString::from_vec(val.into_owned()))
}

pub fn setenv(key: &OsStr, value: &OsStr) -> io::Result<()> {
    let f = |v| Vec::from(OsStr::as_bytes(v)).into();
    args::Env::try_insert(f(key), f(value))
        .map(|_| ())
        .map_err(|_| io::const_io_error!(io::ErrorKind::OutOfMemory, "out of memory"))
}

pub fn unsetenv(key: &OsStr) -> io::Result<()> {
    // Removing non-existent env variables is valid in stdlib.
    let _ = args::Env::remove(OsStr::as_bytes(key));
    Ok(())
}
