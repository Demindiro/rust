use crate::io;
use norostb_rt::kernel::syscall;

pub struct Stdin;
pub struct Stdout;
pub struct Stderr;

impl Stdin {
    pub const fn new() -> Stdin {
        Stdin
    }
}

impl io::Read for Stdin {
    fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
        Ok(0)
    }
}

impl Stdout {
    pub const fn new() -> Stdout {
        Stdout
    }
}

impl io::Write for Stdout {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        syscall::syslog(buf).map_err(|_| {
            io::const_io_error!(io::ErrorKind::Uncategorized, "failed to write to syslog")
        })
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
        syscall::syslog(buf).map_err(|_| {
            io::const_io_error!(io::ErrorKind::Uncategorized, "failed to write to syslog")
        })
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub const STDIN_BUF_SIZE: usize = 0;

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
