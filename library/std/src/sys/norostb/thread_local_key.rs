// TODO move part of this to the runtime crate as other languages will need
// to share the same implementation.

use crate::ptr::NonNull;
use norostb_rt::tls;

pub type Key = usize;

/// # Safety
///
/// This must be called exactly once when a thread is created.
pub(super) unsafe fn init_thread() {
    unsafe {
        tls::init_thread::<_, ()>(|s| {
            Ok(NonNull::new(Box::into_raw(Box::<[u8]>::new_uninit_slice(s)) as *mut *mut ())
                .unwrap())
        }).unwrap_or_else(|_| crate::intrinsics::abort())
    }
}

#[inline]
pub unsafe fn create(dtor: Option<unsafe extern "C" fn(*mut u8)>) -> Key {
    tls::allocate(dtor).expect("failed to allocate TLS slot").0
}

#[inline]
pub unsafe fn set(key: Key, value: *mut u8) {
    unsafe {
        tls::set(tls::Key(key), value);
    }
}

#[inline]
pub unsafe fn get(key: Key) -> *mut u8 {
    unsafe { tls::get(tls::Key(key)) }
}

#[inline]
pub unsafe fn destroy(key: Key) {
    unsafe { tls::free(tls::Key(key)) }
}

#[inline]
pub fn requires_synchronized_create() -> bool {
    false
}
