use crate::alloc::{GlobalAlloc, Layout, System};

macro_rules! imp {
    ($fn:ident($($args:ident : $ty:ty),*) -> $ret:ty) => {
        #[inline]
        unsafe fn $fn(&self, $($args:$ty),*) -> $ret {
            unsafe {
                norostb_rt_alloc::Allocator.$fn($($args),*)
            }
        }
    };
}

#[stable(feature = "alloc_system_type", since = "1.28.0")]
unsafe impl GlobalAlloc for System {
    imp!(alloc(layout: Layout) -> *mut u8);
    imp!(alloc_zeroed(layout: Layout) -> *mut u8);
    imp!(dealloc(ptr: *mut u8, layout: Layout) -> ());
    imp!(realloc(ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8);
}
