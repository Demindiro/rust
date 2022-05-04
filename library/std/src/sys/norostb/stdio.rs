use crate::io;
use norostb_rt as rt;

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
        rt::io::stdin().ok_or(super::ERR_UNSET)?.read(buf).map_err(super::cvt_err)
    }
}

impl Stdout {
    pub const fn new() -> Stdout {
        Stdout
    }
}

impl io::Write for Stdout {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        unsafe { core::arch::asm!("int3") }
        rt::io::stdout().ok_or(super::ERR_UNSET)?.write(buf).map_err(super::cvt_err)
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
        rt::io::stderr().ok_or(super::ERR_UNSET)?.write(buf).map_err(super::cvt_err)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub fn is_ebadf(_err: &io::Error) -> bool {
    true
}

pub fn panic_output() -> Option<impl io::Write> {
    rt::io::stderr().map(|_| Stderr)
}

/// # Safety
///
/// Must be called only once during runtime initialization.
pub(super) unsafe fn init() {}
