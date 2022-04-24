use crate::collections::{btree_map, BTreeMap};
use crate::ffi::{OsStr, OsString};
use crate::fmt;
use crate::io;
use crate::lazy::SyncOnceCell;
use crate::os::norostb::prelude::*;
use crate::ptr::{self, NonNull};
use crate::slice;
use crate::sync::{
    atomic::{AtomicPtr, Ordering},
    Mutex,
};

static ARGS_AND_ENV: AtomicPtr<u8> = AtomicPtr::new(ptr::null_mut());
static ENV: SyncOnceCell<Mutex<BTreeMap<OsString, OsString>>> = SyncOnceCell::new();

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
    inner: btree_map::IntoIter<OsString, OsString>,
}

impl Iterator for Env {
    type Item = (OsString, OsString);

    fn next(&mut self) -> Option<(OsString, OsString)> {
        self.inner.next()
    }
}

fn get_env() -> &'static Mutex<BTreeMap<OsString, OsString>> {
    ENV.get_or_init(|| {
        // A finished args iterator will point to the start of the env variables.
        let mut args = args();
        (&mut args).last();
        // Load all env variables in a map so we can easily modify & remove variables.
        let mut map = BTreeMap::new();
        unsafe {
            let ptr = args.ptr.as_ptr().cast::<u16>();
            let count = usize::from(ptr.read_unaligned());
            let mut ptr = ptr.add(1).cast::<u8>();
            for _ in 0..count {
                let (key, p) = get_str(ptr);
                let (val, p) = get_str(p);
                map.insert(key, val);
                ptr = p;
            }
        }
        Mutex::new(map)
    })
}

pub fn env() -> Env {
    // "The returned iterator contains a snapshot of the process’s environment variables ..." &
    // "Modifications to environment variables afterwards will not be reflected ..."
    // means we need to clone it, or at least use some kind of CoW.
    Env { inner: get_env().lock().expect("failed to lock environment map").clone().into_iter() }
}

pub fn getenv(key: &OsStr) -> Option<OsString> {
    get_env().lock().expect("failed to lock environment map").get(key).cloned()
}

pub fn setenv(key: &OsStr, value: &OsStr) -> io::Result<()> {
    get_env().lock().expect("failed to lock environment map").insert(key.into(), value.into());
    Ok(())
}

pub fn unsetenv(key: &OsStr) -> io::Result<()> {
    // Removing non-existent env variables is valid apparently...
    get_env().lock().expect("failed to lock environment map").remove(key);
    Ok(())
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
