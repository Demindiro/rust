use crate::io;
use norostb_rt::kernel::syscall;

static STDIN: syscall::Handle = syscall::Handle(0);
static STDOUT: syscall::Handle = syscall::Handle(1);
static STDERR: syscall::Handle = syscall::Handle(2);

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
        syscall::read(STDIN, buf)
            .map_err(|_| io::const_io_error!(io::ErrorKind::Uncategorized, "failed to read"))
    }
}

impl Stdout {
    pub const fn new() -> Stdout {
        Stdout
    }
}

impl io::Write for Stdout {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        syscall::write(STDOUT, buf)
            .map_err(|_| io::const_io_error!(io::ErrorKind::Uncategorized, "failed to write"))
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
        syscall::write(STDERR, buf)
            .map_err(|_| io::const_io_error!(io::ErrorKind::Uncategorized, "failed to write"))
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub fn is_ebadf(_err: &io::Error) -> bool {
    true
}

pub fn panic_output() -> Option<impl io::Write> {
    Some(syscall::SysLog::default())
}

#[unstable(feature = "norostb", issue = "none")]
impl io::Write for syscall::SysLog {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.write_raw(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(self.flush())
    }
}

/// # Safety
///
/// Must be called only once during runtime initialization.
pub(super) unsafe fn init() {
    use syscall::{Id, TableId};
    let stdin = syscall::open(TableId(0), Id(0)).unwrap();
    assert_eq!(stdin, STDIN);
    let stdout = syscall::open(TableId(0), Id(0)).unwrap();
    assert_eq!(stdout, STDOUT);
    let stderr = syscall::open(TableId(0), Id(0)).unwrap();
    assert_eq!(stderr, STDERR);
}
