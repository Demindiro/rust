use crate::{io, sync::atomic::Ordering};
use norostb_rt::kernel::AtomicHandle;

static STDIN: AtomicHandle = AtomicHandle::new(0);
static STDOUT: AtomicHandle = AtomicHandle::new(0);
static STDERR: AtomicHandle = AtomicHandle::new(0);

pub struct Stdin;
pub struct Stdout;
pub struct Stderr;

pub const STDIN_BUF_SIZE: usize = 512;

impl Stdin {
    pub const fn new() -> Stdin {
        Stdin
    }
}

impl io::Read for Stdin {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        super::io::read(STDIN.load(Ordering::Relaxed), buf)
    }
}

impl Stdout {
    pub const fn new() -> Stdout {
        Stdout
    }
}

impl io::Write for Stdout {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        super::io::write(STDOUT.load(Ordering::Relaxed), buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Stderr {
    pub const fn new() -> Stderr {
        Stderr
    }
}

impl io::Write for Stderr {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        super::io::write(STDERR.load(Ordering::Relaxed), buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub fn is_ebadf(_err: &io::Error) -> bool {
    true
}

pub fn panic_output() -> Option<impl io::Write> {
    Some(Stderr)
}

/// # Safety
///
/// Must be called only once during runtime initialization.
pub(super) unsafe fn init() {
    // Find the UART table
    let mut tbl = None;
    for (id, info) in super::io::tables() {
        if info.name() == b"uart" {
            tbl = Some(id);
            break;
        }
    }
    // If we couldn't find the table, there is absolutely nothing we can do, so just abort.
    let tbl = tbl.unwrap_or_else(|| core::intrinsics::abort());

    STDIN.store(super::io::open(tbl, b"0").unwrap(), Ordering::Relaxed);
    STDOUT.store(super::io::open(tbl, b"0").unwrap(), Ordering::Relaxed);
    STDERR.store(super::io::open(tbl, b"0").unwrap(), Ordering::Relaxed);
}
