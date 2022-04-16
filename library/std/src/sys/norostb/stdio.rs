use crate::io;
use norostb_rt::kernel::syscall;

static STDIN: syscall::Handle = 0;
static STDOUT: syscall::Handle = 1;
static STDERR: syscall::Handle = 2;

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
        super::io::read(STDIN, buf)
    }
}

impl Stdout {
    pub const fn new() -> Stdout {
        Stdout
    }
}

impl io::Write for Stdout {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        super::io::write(STDOUT, buf)
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
        super::io::write(STDERR, buf)
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

    let stdin = super::io::open(tbl, b"0").unwrap();
    assert_eq!(stdin, STDIN);
    let stdout = super::io::open(tbl, b"0").unwrap();
    assert_eq!(stdout, STDOUT);
    let stderr = super::io::open(tbl, b"0").unwrap();
    assert_eq!(stderr, STDERR);
}
