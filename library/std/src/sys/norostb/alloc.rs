use crate::alloc::{GlobalAlloc, Layout, System};

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
    unsafe fn realloc(&self, _ptr: *mut u8, _layout: Layout, _new_size: usize) -> *mut u8 {
        0 as *mut u8
    }
}
