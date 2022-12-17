use std::time::Duration;

use crate::{Level, Timestamp};

pub enum AdsrEventType {
    NoteOn,
    NoteOff,

    SetAttack(Duration),
    SetDecay(Duration),
    SetSustain(Level),
    SetRelease(Duration),
}

pub struct AdsrEvent {
    pub time: Timestamp,
    pub event_type: AdsrEventType,
}
