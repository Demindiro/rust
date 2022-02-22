use crate::ptr;

pub type Key = usize;

// FIXME implement real TLS
static mut TLS: Vec<(*mut u8, Option<unsafe extern "C" fn(*mut u8)>)> = Vec::new();

#[inline]
pub unsafe fn create(dtor: Option<unsafe extern "C" fn(*mut u8)>) -> Key {
    unsafe {
        TLS.push((ptr::null_mut(), dtor));
        TLS.len() - 1
    }
}

#[inline]
pub unsafe fn set(key: Key, value: *mut u8) {
    // TODO do we have to call the destructor on the old value?
    unsafe {
        debug_assert!(key < TLS.len());
        TLS.get_unchecked_mut(key).0 = value;
    }
}

#[inline]
pub unsafe fn get(key: Key) -> *mut u8 {
    unsafe {
        debug_assert!(key < TLS.len());
        TLS.get_unchecked(key).0
    }
}

#[inline]
pub unsafe fn destroy(key: Key) {
    unsafe {
        debug_assert!(key < TLS.len());
        let p = TLS.get_unchecked(key);
        if let Some(f) = p.1 {
            if !p.0.is_null() {
                f(p.0);
            }
        }
        let _ = p;
    }
}

#[inline]
pub fn requires_synchronized_create() -> bool {
    false
}
