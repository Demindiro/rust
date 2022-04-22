use crate::cell::RefCell;
use crate::io;
use crate::mem::{self, MaybeUninit};
use norostb_rt::kernel::{
    io::{Job, ObjectInfo, Queue, Request, Response, SeekFrom},
    syscall::{self, TableId, TableInfo},
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
        Queue {
            base: syscall::create_io_queue(None, 0, 0)
                .unwrap_or_else(|e| rtabort!("failed to create io queue: {:?}", e))
                .cast(),
            requests_mask: 0,
            responses_mask: 0,
        }
    });
}

fn enqueue(request: Request) -> Response {
    QUEUE.with(|queue| unsafe {
        let mut queue = queue.borrow_mut();
        queue.enqueue_request(request).unwrap();
        syscall::process_io_queue(Some(queue.base.cast())).unwrap();
        loop {
            if let Ok(e) = queue.dequeue_response() {
                break e;
            }
            syscall::wait_io_queue(Some(queue.base.cast())).unwrap();
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
pub fn open(table: syscall::TableId, path: &[u8]) -> io::Result<syscall::Handle> {
    let e = enqueue(Request::open(0, table, path));
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
pub fn create(table: syscall::TableId, path: &[u8]) -> io::Result<syscall::Handle> {
    let e = enqueue(Request::create(0, table, path));
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
pub fn query(table: syscall::TableId, path: &[u8]) -> io::Result<syscall::Handle> {
    let e = enqueue(Request::query(0, table, path));
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
pub fn query_next(query: syscall::Handle, info: &mut ObjectInfo) -> io::Result<bool> {
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
pub fn take_job(table: syscall::Handle, job: &mut Job) -> io::Result<()> {
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
pub fn finish_job(table: syscall::Handle, job: &Job) -> io::Result<()> {
    let e = enqueue(Request::finish_job(0, table, &job));
    if e.value < 0 {
        Err(io::const_io_error!(io::ErrorKind::Uncategorized, "failed to finish job"))
    } else {
        Ok(())
    }
}

/// Blocking seek
#[unstable(feature = "norostb", issue = "none")]
#[inline]
pub fn seek(handle: syscall::Handle, from: io::SeekFrom) -> io::Result<u64> {
    let mut offset = 0;
    let from = match from {
        io::SeekFrom::Start(n) => SeekFrom::Start(n),
        io::SeekFrom::End(n) => SeekFrom::End(n),
        io::SeekFrom::Current(n) => SeekFrom::Current(n),
    };
    let e = enqueue(Request::seek(0, handle, from, &mut offset));
    if e.value < 0 {
        Err(io::const_io_error!(io::ErrorKind::Uncategorized, "failed to seek"))
    } else {
        Ok(offset)
    }
}

/// Blocking poll
#[unstable(feature = "norostb", issue = "none")]
#[inline]
pub fn poll(handle: syscall::Handle) -> io::Result<usize> {
    let e = enqueue(Request::poll(0, handle));
    if e.value < 0 {
        Err(io::const_io_error!(io::ErrorKind::Uncategorized, "failed to poll"))
    } else {
        Ok(e.value as usize)
    }
}

/// Blocking close
#[unstable(feature = "norostb", issue = "none")]
#[inline]
pub fn close(handle: syscall::Handle) {
    enqueue(Request::close(0, handle));
}

/// Create an iterator over all tables.
#[unstable(feature = "norostb", issue = "none")]
#[inline]
pub fn tables() -> TableIter {
    TableIter { state: Some(None) }
}

/// An iterator over all tables.
#[derive(Clone, Debug)]
#[unstable(feature = "norostb", issue = "none")]
pub struct TableIter {
    state: Option<Option<TableId>>,
}

impl Iterator for TableIter {
    type Item = (TableId, TableInfo);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.state.take().and_then(|id| syscall::next_table(id)).map(|(id, info)| {
            self.state = Some(Some(id));
            (id, info)
        })
    }
}
