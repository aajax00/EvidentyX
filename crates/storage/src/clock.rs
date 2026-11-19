//! Source de l'heure courante, injectable pour des tests déterministes.

use evidentyx_core::Timestamp;
use std::time::{SystemTime, UNIX_EPOCH};

pub trait Clock: Send {
    fn now(&self) -> Timestamp;
}

/// Horloge système (UTC).
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Timestamp {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX))
            // Horloge système antérieure à 1970 : on se replie sur l'époque.
            .unwrap_or(0);
        Timestamp::from_unix_millis(millis)
    }
}
