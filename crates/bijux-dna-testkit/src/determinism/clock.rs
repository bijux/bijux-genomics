use std::time::{Duration, SystemTime};

#[derive(Debug, Clone)]
pub struct FixedClock {
    now: SystemTime,
}

impl FixedClock {
    #[must_use]
    pub fn at(now: SystemTime) -> Self {
        Self { now }
    }

    #[must_use]
    pub fn unix_s(secs: u64) -> Self {
        Self {
            now: SystemTime::UNIX_EPOCH + Duration::from_secs(secs),
        }
    }

    #[must_use]
    pub fn now(&self) -> SystemTime {
        self.now
    }
}
