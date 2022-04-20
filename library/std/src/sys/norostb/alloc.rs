use crate::alloc::{Allocator as Alloc, GlobalAlloc, Layout, System};
use crate::ptr::{self, NonNull};
use crate::sys_common::mutex::StaticMutex;
use norostb_rt::alloc::Allocator;

static ALLOCATOR_LOCK: StaticMutex = StaticMutex::new();
static mut ALLOCATOR: Allocator = Allocator::new();

#[stable(feature = "alloc_system_type", since = "1.28.0")]
unsafe impl GlobalAlloc for System {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe {
            let _guard = ALLOCATOR_LOCK.lock();
            ALLOCATOR.allocate(layout).map_or(ptr::null_mut(), |p| p.as_ptr().as_mut_ptr())
        }
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        unsafe {
            let _guard = ALLOCATOR_LOCK.lock();
            ALLOCATOR.allocate_zeroed(layout).map_or(ptr::null_mut(), |p| p.as_ptr().as_mut_ptr())
        }
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        debug_assert!(!ptr.is_null());
        unsafe {
            let _guard = ALLOCATOR_LOCK.lock();
            ALLOCATOR.deallocate(NonNull::new_unchecked(ptr), layout)
        }
    }

    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        debug_assert!(!ptr.is_null());
        let old_layout = layout;
        if let Ok(new_layout) = Layout::from_size_align(new_size, layout.align()) {
            unsafe {
                let _guard = ALLOCATOR_LOCK.lock();
                if new_layout.size() > old_layout.size() {
                    ALLOCATOR
                        .grow(NonNull::new_unchecked(ptr), old_layout, new_layout)
                        .map_or(ptr::null_mut(), |p| p.as_ptr().as_mut_ptr())
                } else {
                    ALLOCATOR
                        .shrink(NonNull::new_unchecked(ptr), old_layout, new_layout)
                        .map_or(ptr::null_mut(), |p| p.as_ptr().as_mut_ptr())
                }
            }
        } else {
            ptr::null_mut()
        }
    }
}
