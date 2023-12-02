use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Copy, Serialize, Deserialize, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(C)]
pub struct Duration(pub time::Duration);

impl Duration {

    pub const fn from_minutes(minutes: i64) -> Self {
        Self(time::Duration::minutes(minutes))
    }

    pub const fn from_seconds(seconds: i64) -> Self {
        Self(time::Duration::seconds(seconds))
    }

    pub const fn from_milliseconds(milliseconds: i64) -> Self {
        Self(time::Duration::milliseconds(milliseconds))
    }

    pub const fn from_microseconds(microseconds: i64) -> Self {
        Self(time::Duration::microseconds(microseconds))
    }

    pub const fn from_nanoseconds(nanoseconds: i64) -> Self {
        Self(time::Duration::nanoseconds(nanoseconds))
    }

    pub fn seconds(self) -> i64 {
        self.0.whole_seconds()
    }

    pub fn milliseconds(self) -> i128 {
        self.0.whole_milliseconds()
    }

    pub fn microseconds(self) -> i128 {
        self.0.whole_microseconds()
    }

    pub fn nanoseconds(self) -> i128 {
        self.0.whole_nanoseconds()
    }

    pub fn days(self) -> i64 {
        self.0.whole_days()
    }

    pub fn hours(self) -> i64 {
        self.0.whole_hours()
    }

    pub fn weeks(self) -> i64 {
        self.0.whole_weeks()
    }

    pub(crate) fn to_std(self) -> std::time::Duration {
        let whole_seconds = self.0.whole_seconds() as u64;
        let subsec_nanos = self.0.subsec_nanoseconds() as u32;
        std::time::Duration::new(whole_seconds, subsec_nanos)
    }
}
