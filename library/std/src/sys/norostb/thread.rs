use super::unsupported;
use crate::ffi::CStr;
use crate::io;
use crate::num::NonZeroUsize;
use crate::time::Duration;
use norostb_rt as rt;

pub struct Thread(rt::thread::Thread);

pub const DEFAULT_MIN_STACK_SIZE: usize = 1 << 16;

impl Thread {
    // unsafe: see thread::Builder::spawn_unchecked for safety requirements
    pub unsafe fn new(stack: usize, p: Box<dyn FnOnce()>) -> io::Result<Self> {
        unsafe { rt::thread::Thread::new(stack, p).map_err(super::cvt_err).map(Self) }
    }

    pub fn yield_now() {
        rt::thread::sleep(Duration::ZERO);
    }

    pub fn set_name(_name: &CStr) {
        // nope
    }

    pub fn sleep(dur: Duration) {
        rt::thread::sleep(dur);
    }

    pub fn join(self) {
        self.0.wait()
    }
}

pub fn available_parallelism() -> io::Result<NonZeroUsize> {
    unsupported()
}

pub mod guard {
    pub type Guard = !;
    pub unsafe fn current() -> Option<Guard> {
        None
    }
    pub unsafe fn init() -> Option<Guard> {
        None
    }
}
