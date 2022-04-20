use super::unsupported;
use crate::ffi::CStr;
use crate::io;
use crate::mem;
use crate::num::NonZeroUsize;
use crate::ptr;
use crate::time::Duration;
use norostb_rt::kernel::syscall::{self, RWX};

pub struct Thread {
    handle: usize,
}

pub const DEFAULT_MIN_STACK_SIZE: usize = 4096;

impl Thread {
    // unsafe: see thread::Builder::spawn_unchecked for safety requirements
    pub unsafe fn new(stack: usize, p: Box<dyn FnOnce()>) -> io::Result<Thread> {
        // Allocate stack
        let (stack, stack_size) = syscall::alloc(None, stack, RWX::RW).map_err(|_| {
            io::const_io_error!(io::ErrorKind::Uncategorized, "failed to allocate stack space")
        })?;
        let stack = stack.cast::<u8>();

        // Push closure on the stack of the new thread
        let (ptr, meta) = Box::into_raw(p).to_raw_parts();
        let stack_top = stack.as_ptr().wrapping_add(stack_size.get()).cast::<usize>();
        let mut stack_ptr = stack_top;
        let mut push = |v: usize| {
            stack_ptr = stack_ptr.wrapping_sub(1);
            // SAFETY: we will only push two usizes, which should fit well within a single
            // page.
            unsafe {
                stack_ptr.write(v);
            }
        };
        push(ptr as usize);
        push(unsafe { mem::transmute(meta) });

        unsafe extern "C" fn main(ptr: *mut (), meta: usize) -> ! {
            let meta = unsafe { mem::transmute(meta) };
            let p: Box<dyn FnOnce()> = unsafe { Box::from_raw(ptr::from_raw_parts_mut(ptr, meta)) };

            unsafe {
                super::thread_local_key::init_thread();
            }

            p();

            loop {
                syscall::sleep(crate::time::Duration::MAX);
            }
        }

        #[naked]
        unsafe extern "C" fn start() -> ! {
            unsafe {
                crate::arch::asm!("
					mov rdi, [rsp - 8 * 1]
					mov rsi, [rsp - 8 * 2]
					jmp {main}
					",
                    main = sym main,
                    options(noreturn),
                );
            }
        }

        // Spawn thread
        unsafe {
            syscall::spawn_thread(start, stack_top as *const ())
                .map_err(|_| io::const_io_error!(io::ErrorKind::Other, "failed to spawn thread"))
                .map(|handle| Self { handle })
        }
    }

    pub fn yield_now() {
        syscall::sleep(Duration::ZERO);
    }

    pub fn set_name(_name: &CStr) {
        // nope
    }

    pub fn sleep(dur: Duration) {
        syscall::sleep(dur);
    }

    pub fn join(self) {
        todo!("join thread {}", self.handle);
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
