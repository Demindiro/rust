use crate::time::Duration;
use norostb_rt::time::Monotonic;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct Instant(Monotonic);

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct SystemTime(Duration);

pub const UNIX_EPOCH: SystemTime = SystemTime(Duration::from_secs(0));

impl Instant {
    pub fn now() -> Instant {
        Self(Monotonic::now())
    }

    pub fn checked_sub_instant(&self, other: &Instant) -> Option<Duration> {
        Duration::from_nanos(self.0.as_nanos())
            .checked_sub(Duration::from_nanos(other.0.as_nanos()))
    }

    pub fn checked_add_duration(&self, other: &Duration) -> Option<Instant> {
        Duration::from_nanos(self.0.as_nanos())
            .checked_add(*other)?
            .as_nanos()
            .try_into()
            .ok()
            .map(Monotonic::from_nanos)
            .map(Self)
    }

    pub fn checked_sub_duration(&self, other: &Duration) -> Option<Instant> {
        Duration::from_nanos(self.0.as_nanos())
            .checked_sub(*other)?
            .as_nanos()
            .try_into()
            .ok()
            .map(Monotonic::from_nanos)
            .map(Self)
    }
}

impl SystemTime {
    pub fn now() -> SystemTime {
        panic!("time not implemented on this platform")
    }

    pub fn sub_time(&self, other: &SystemTime) -> Result<Duration, Duration> {
        self.0.checked_sub(other.0).ok_or_else(|| other.0 - self.0)
    }

    pub fn checked_add_duration(&self, other: &Duration) -> Option<SystemTime> {
        Some(SystemTime(self.0.checked_add(*other)?))
    }

    pub fn checked_sub_duration(&self, other: &Duration) -> Option<SystemTime> {
        Some(SystemTime(self.0.checked_sub(*other)?))
    }
}
