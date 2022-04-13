use crate::cell::RefCell;
use crate::io;
use crate::mem::{self, MaybeUninit};
use norostb_rt::kernel::{
    io::{Queue, Request, Response},
    syscall,
};

#[derive(Copy, Clone)]
pub struct IoSlice<'a>(&'a [u8]);

impl<'a> IoSlice<'a> {
    #[inline]
    pub fn new(buf: &'a [u8]) -> IoSlice<'a> {
        IoSlice(buf)
    }

    #[inline]
    pub fn advance(&mut self, n: usize) {
        self.0 = &self.0[n..]
    }

    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        self.0
    }
}

pub struct IoSliceMut<'a>(&'a mut [u8]);

impl<'a> IoSliceMut<'a> {
    #[inline]
    pub fn new(buf: &'a mut [u8]) -> IoSliceMut<'a> {
        IoSliceMut(buf)
    }

    #[inline]
    pub fn advance(&mut self, n: usize) {
        let slice = mem::replace(&mut self.0, &mut []);
        let (_, remaining) = slice.split_at_mut(n);
        self.0 = remaining;
    }

    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        self.0
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        self.0
    }
}

thread_local! {
    static QUEUE: RefCell<Queue> = RefCell::new({
        use crate::sync::atomic::*;
        static ADDR: AtomicUsize = AtomicUsize::new(0x9_8765_0000);
        let base = ADDR.fetch_add(0x1000, Ordering::Relaxed);
        let base = syscall::create_io_queue(base as *mut _, 0, 0).unwrap();
        let base = crate::ptr::NonNull::new(base).unwrap().cast();
        Queue {
            base,
            requests_mask: 0,
            responses_mask: 0,
        }
    });
}

fn enqueue(request: Request) -> Response {
    QUEUE.with(|queue| unsafe {
        let mut queue = queue.borrow_mut();
        queue.enqueue_request(request).unwrap();
        let base = queue.base.as_ptr().cast();
        syscall::process_io_queue(base).unwrap();
        loop {
            if let Ok(e) = queue.dequeue_response() {
                break e;
            }
            syscall::wait_io_queue(base).unwrap();
        }
    })
}

/// Blocking read
#[unstable(feature = "norostb", issue = "none")]
#[inline]
pub fn read(handle: syscall::Handle, data: &mut [u8]) -> io::Result<usize> {
    unsafe { read_uninit(handle, mem::transmute(data)) }
}

/// Blocking read
#[unstable(feature = "norostb", issue = "none")]
#[inline]
pub fn read_uninit(handle: syscall::Handle, data: &mut [MaybeUninit<u8>]) -> io::Result<usize> {
    let e = enqueue(Request::read_uninit(0, handle, data));
    if e.value < 0 {
        Err(io::const_io_error!(io::ErrorKind::Uncategorized, "failed to read"))
    } else {
        Ok(e.value as usize)
    }
}

/// Blocking write
#[unstable(feature = "norostb", issue = "none")]
#[inline]
pub fn write(handle: syscall::Handle, data: &[u8]) -> io::Result<usize> {
    let e = enqueue(Request::write(0, handle, data));
    if e.value < 0 {
        Err(io::const_io_error!(io::ErrorKind::Uncategorized, "failed to write"))
    } else {
        Ok(e.value as usize)
    }
}

/// Blocking open
#[unstable(feature = "norostb", issue = "none")]
#[inline]
pub fn open(table: syscall::TableId, object: syscall::Id) -> io::Result<syscall::Handle> {
    let e = enqueue(Request::open(0, table, object));
    if e.value < 0 {
        Err(io::const_io_error!(io::ErrorKind::Uncategorized, "failed to open"))
    } else {
        // This error pops up despite stdlib already using 2021 edition?!?
        /*
            = help: items from traits can only be used if the trait is in scope
            = note: 'core::convert::TryInto' is included in the prelude starting in Edition 2021
        help: the following trait is implemented but not in scope; perhaps add a `use` for it:
                */
        use crate::convert::TryInto;
        Ok(e.value.try_into().unwrap())
    }
}

/// Blocking create
#[unstable(feature = "norostb", issue = "none")]
#[inline]
pub fn create(table: syscall::TableId, tags: &[u8]) -> io::Result<syscall::Handle> {
    let e = enqueue(Request::create(0, table, tags));
    if e.value < 0 {
        Err(io::const_io_error!(io::ErrorKind::Uncategorized, "failed to create"))
    } else {
        // This error pops up despite stdlib already using 2021 edition?!?
        /*
            = help: items from traits can only be used if the trait is in scope
            = note: 'core::convert::TryInto' is included in the prelude starting in Edition 2021
        help: the following trait is implemented but not in scope; perhaps add a `use` for it:
                */
        use crate::convert::TryInto;
        Ok(e.value.try_into().unwrap())
    }
}

/// Blocking query
#[unstable(feature = "norostb", issue = "none")]
#[inline]
pub fn query(table: syscall::TableId, tags: &[u8]) -> io::Result<syscall::QueryHandle> {
    let e = enqueue(Request::query(0, table, tags));
    if e.value < 0 {
        Err(io::const_io_error!(io::ErrorKind::Uncategorized, "failed to query"))
    } else {
        // This error pops up despite stdlib already using 2021 edition?!?
        /*
            = help: items from traits can only be used if the trait is in scope
            = note: 'core::convert::TryInto' is included in the prelude starting in Edition 2021
        help: the following trait is implemented but not in scope; perhaps add a `use` for it:
                */
        use crate::convert::TryInto;
        Ok(e.value.try_into().unwrap())
    }
}

/// Blocking query_next
#[unstable(feature = "norostb", issue = "none")]
#[inline]
pub fn query_next(query: syscall::QueryHandle, info: &mut syscall::ObjectInfo) -> io::Result<bool> {
    let e = enqueue(Request::query_next(0, query, info));
    if e.value < 0 {
        Err(io::const_io_error!(io::ErrorKind::Uncategorized, "failed to advance query"))
    } else {
        Ok(e.value > 0)
    }
}

/// Blocking take_job
#[unstable(feature = "norostb", issue = "none")]
#[inline]
pub fn take_job(table: syscall::Handle, job: &mut syscall::Job) -> io::Result<()> {
    let e = enqueue(Request::take_job(0, table, job));
    if e.value < 0 {
        Err(io::const_io_error!(io::ErrorKind::Uncategorized, "failed to take job"))
    } else {
        Ok(())
    }
}

/// Blocking finish_job
#[unstable(feature = "norostb", issue = "none")]
#[inline]
pub fn finish_job(table: syscall::Handle, job: &syscall::Job) -> io::Result<()> {
    let e = enqueue(Request::finish_job(0, table, &job));
    if e.value < 0 {
        Err(io::const_io_error!(io::ErrorKind::Uncategorized, "failed to finish job"))
    } else {
        Ok(())
    }
}
