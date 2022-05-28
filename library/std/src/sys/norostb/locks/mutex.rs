use norostb_rt::sync::RawMutex;

pub struct Mutex(RawMutex);

pub type MovableMutex = Mutex;

impl Mutex {
    pub const fn new() -> Mutex {
        Mutex(RawMutex::new())
    }

    #[inline]
    pub unsafe fn init(&mut self) {}

    #[inline]
    pub unsafe fn lock(&self) {
        self.0.lock();
    }

    #[inline]
    pub unsafe fn unlock(&self) {
        self.0.unlock()
    }

    #[inline]
    pub unsafe fn try_lock(&self) -> bool {
        self.0.try_lock()
    }

    #[inline]
    pub unsafe fn destroy(&self) {}
}
