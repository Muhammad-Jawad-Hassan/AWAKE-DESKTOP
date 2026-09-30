//! Serializes a `Duration` as whole seconds.

use std::time::Duration;

use serde::{Deserialize, Deserializer, Serializer};

pub fn serialize<S: Serializer>(duration: &Duration, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_u64(duration.as_secs())
}

pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Duration, D::Error> {
    u64::deserialize(d).map(Duration::from_secs)
}
