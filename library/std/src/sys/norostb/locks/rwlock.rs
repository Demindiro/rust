use norostb_rt::sync::RawRwLock;

pub struct RwLock(RawRwLock);

pub type MovableRwLock = RwLock;

unsafe impl Sync for RwLock {} // no threads on this platform

impl RwLock {
    pub const fn new() -> RwLock {
        RwLock(RawRwLock::new())
    }

    #[inline]
    pub unsafe fn read(&self) {
        self.0.read()
    }

    #[inline]
    pub unsafe fn try_read(&self) -> bool {
        self.0.try_read()
    }

    #[inline]
    pub unsafe fn write(&self) {
        self.0.write()
    }

    #[inline]
    pub unsafe fn try_write(&self) -> bool {
        self.0.try_write()
    }

    #[inline]
    pub unsafe fn read_unlock(&self) {
        self.0.read_unlock()
    }

    #[inline]
    pub unsafe fn write_unlock(&self) {
        self.0.write_unlock()
    }

    #[inline]
    pub unsafe fn destroy(&self) {}
}
