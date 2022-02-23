use crate::io;

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
        let status @ len: usize;
        unsafe {
            crate::arch::asm!(
                "syscall",
                in("eax") 0,
                in("rdi") buf.as_ptr(),
                in("rsi") buf.len(),
                lateout("rax") status,
                lateout("rdx") len,
                lateout("rcx") _,
                lateout("r11") _,
            );
        }
        (status == 0)
            .then(|| len)
            .ok_or(io::const_io_error!(io::ErrorKind::Uncategorized, "failed to write to syslog"))
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
        let status @ len: usize;
        unsafe {
            crate::arch::asm!(
                "syscall",
                in("eax") 0,
                in("rdi") buf.as_ptr(),
                in("rsi") buf.len(),
                lateout("rax") status,
                lateout("rdx") len,
                lateout("rcx") _,
                lateout("r11") _,
            );
        }
        (status == 0)
            .then(|| len)
            .ok_or(io::const_io_error!(io::ErrorKind::Uncategorized, "failed to write to syslog"))
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
    Some(Stderr::new())
}
