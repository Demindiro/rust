use crate::ffi::{OsStr, OsString};
use crate::fmt;
use crate::io;
use crate::os::norostb::prelude::*;
use crate::ptr::{self, NonNull};
use crate::slice;
use crate::sync::atomic::{AtomicPtr, Ordering};

static ARGS_AND_ENV: AtomicPtr<u8> = AtomicPtr::new(ptr::null_mut());
static ENV: AtomicPtr<u8> = AtomicPtr::new(ptr::null_mut());

pub struct Args {
    count: usize,
    ptr: NonNull<u8>,
}

pub fn args() -> Args {
    unsafe {
        let ptr = NonNull::new(ARGS_AND_ENV.load(Ordering::Relaxed))
            .expect("No arguments were set")
            .cast::<u16>();
        Args {
            count: usize::from(ptr.as_ptr().read_unaligned()),
            ptr: NonNull::new(ptr.as_ptr().add(1).cast()).unwrap(),
        }
    }
}

impl fmt::Debug for Args {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let args = Args { count: self.count, ptr: self.ptr };
        let mut f = f.debug_list();
        for e in args {
            f.entry(&e);
        }
        f.finish()
    }
}

impl Iterator for Args {
    type Item = OsString;

    fn next(&mut self) -> Option<OsString> {
        self.count.checked_sub(1).map(|c| {
            self.count = c;
            unsafe {
                let (val, ptr) = get_str(self.ptr.as_ptr());
                self.ptr = NonNull::new(ptr).unwrap();
                val
            }
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.count, Some(self.count))
    }
}

impl ExactSizeIterator for Args {
    fn len(&self) -> usize {
        self.count
    }
}

impl DoubleEndedIterator for Args {
    fn next_back(&mut self) -> Option<OsString> {
        self.count.checked_sub(1).map(|c| {
            // Very inefficient but w/e, it shouldn't matter.
            let args = Args { count: self.count, ptr: self.ptr };
            self.count = c;
            args.last().unwrap()
        })
    }
}

pub struct Env {
    count: usize,
    ptr: NonNull<u8>,
}

impl fmt::Debug for Env {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let env = Env { count: self.count, ptr: self.ptr };
        let mut f = f.debug_map();
        for (k, v) in env {
            f.entry(&k, &v);
        }
        f.finish()
    }
}

impl Iterator for Env {
    type Item = (OsString, OsString);

    fn next(&mut self) -> Option<(OsString, OsString)> {
        self.count.checked_sub(1).map(|c| {
            self.count = c;
            unsafe {
                let (key, ptr) = get_str(self.ptr.as_ptr());
                let (val, ptr) = get_str(ptr);
                self.ptr = NonNull::new(ptr).unwrap();
                (key, val)
            }
        })
    }
}

pub fn env() -> Env {
    let ptr = NonNull::new(ENV.load(Ordering::Relaxed))
        .unwrap_or_else(|| {
            // A finished args iterator will point to the start of the env variables.
            let mut args = args();
            (&mut args).last();
            ENV.store(args.ptr.as_ptr(), Ordering::Relaxed);
            args.ptr
        })
        .cast::<u16>();
    unsafe {
        Env {
            count: usize::from(ptr.as_ptr().read_unaligned()),
            ptr: NonNull::new(ptr.as_ptr().add(1).cast()).unwrap(),
        }
    }
}

pub fn getenv(key: &OsStr) -> Option<OsString> {
    env().find_map(|(k, v)| (k == key).then(|| v))
}

pub fn setenv(_: &OsStr, _: &OsStr) -> io::Result<()> {
    todo!()
}

pub fn unsetenv(_: &OsStr) -> io::Result<()> {
    todo!()
}

/// # Safety
///
/// Must be called only once during runtime initialization.
pub(super) unsafe fn init(args_and_env: *const u8) {
    ARGS_AND_ENV.store(args_and_env as _, Ordering::Relaxed)
}

unsafe fn get_str(ptr: *mut u8) -> (OsString, *mut u8) {
    let len = usize::from(unsafe { ptr.cast::<u16>().read_unaligned() });
    let ptr = ptr.wrapping_add(2);
    let mut s = OsString::new();
    s.push(OsStr::from_bytes(unsafe { slice::from_raw_parts(ptr, len) }));
    (s, ptr.wrapping_add(len))
}
