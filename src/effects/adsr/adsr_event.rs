use std::time::Duration;

use crate::{effects::utility::EventProcessorEvent, Level, Timestamp};

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

impl EventProcessorEvent for AdsrEvent {
    fn get_time(&self) -> Timestamp {
        self.time
    }

    fn should_clear_queue(&self) -> bool {
        false
    }
}
