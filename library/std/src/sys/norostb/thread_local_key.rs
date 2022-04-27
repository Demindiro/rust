// TODO move part of this to the runtime crate as other languages will need
// to share the same implementation.

use norostb_rt::tls;

pub type Key = usize;

#[inline]
pub unsafe fn create(dtor: Option<unsafe extern "C" fn(*mut u8)>) -> Key {
    let dtor = unsafe { core::mem::transmute(dtor) };
    tls::allocate(dtor).expect("failed to allocate TLS slot").0
}

#[inline]
pub unsafe fn set(key: Key, value: *mut u8) {
    unsafe {
        tls::set(tls::Key(key), value.cast());
    }
}

#[inline]
pub unsafe fn get(key: Key) -> *mut u8 {
    unsafe { tls::get(tls::Key(key)).cast() }
}

#[inline]
pub unsafe fn destroy(key: Key) {
    unsafe { tls::free(tls::Key(key)) }
}

#[inline]
pub fn requires_synchronized_create() -> bool {
    false
}
