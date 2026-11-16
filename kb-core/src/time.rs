//! Points in time and the clock the facade reads them from.

use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// A point in time: milliseconds since 1970-01-01T00:00:00Z (Unix epoch, UTC), without
/// leap seconds. In JSON and TypeScript a plain number (`Date` takes it directly), so
/// there is no time zone to get wrong; local time is a matter of display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct Timestamp(pub u64);

impl Timestamp {
    /// Converts a system time; times before the epoch become 0, the sub-millisecond
    /// part is dropped. Used for file modification times as well.
    pub fn from_system_time(time: SystemTime) -> Self {
        let millis = time
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        Self(u64::try_from(millis).unwrap_or(u64::MAX))
    }
}

/// Source of the current time.
///
/// The [`KnowledgeBase`](crate::KnowledgeBase) reads the time for chat timestamps from
/// its clock, [`SystemClock`] by default; tests set a fixed clock instead, so they do
/// not depend on the real time.
pub trait Clock: Send + Sync {
    /// The current time.
    fn now(&self) -> Timestamp;
}

/// The operating system clock.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Timestamp {
        Timestamp::from_system_time(SystemTime::now())
    }
}
