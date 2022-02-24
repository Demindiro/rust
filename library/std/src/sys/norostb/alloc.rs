use crate::alloc::{GlobalAlloc, Layout, System};
use crate::ptr;

#[repr(align(16))]
#[derive(Clone, Copy)]
struct E([u8; 16]);
static mut HEAP: [E; 4096] = [E([0; 16]); 4096];
static mut HEAP_I: usize = 0;

#[stable(feature = "alloc_system_type", since = "1.28.0")]
unsafe impl GlobalAlloc for System {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe { self.alloc_zeroed(layout) }
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        unsafe {
            let s = layout.pad_to_align().size();
            let s = (s + 15) & !15;
            let p = HEAP.as_ptr().add(HEAP_I);
            HEAP_I += s / 16;
            p as *mut u8
        }
    }

    #[inline]
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}

    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        if layout.size() >= new_size {
            return ptr;
        }
        // SAFETY: the caller has to ensure new_size doesn't overflow
        let new_layout = unsafe { Layout::from_size_align_unchecked(new_size, layout.align()) };
        // SAFETY: the caller has to ensure new_size is non-zero
        let new_ptr = unsafe { self.alloc_zeroed(new_layout) };
        if !new_ptr.is_null() {
            // SAFETY: the old and new pointer are both valid and cannot overlap.
            unsafe {
                ptr::copy_nonoverlapping(ptr, new_ptr, layout.size().min(new_size));
            }
        }
        new_ptr
    }
}
