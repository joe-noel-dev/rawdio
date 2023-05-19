use std::{
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};

use crate::{effects::utility::EventProcessorEvent, Level, Timestamp};

#[derive(PartialEq, PartialOrd)]
pub enum AdsrEventType {
    NoteOff,
    NoteOn,

    SetAttack(Duration),
    SetDecay(Duration),
    SetSustain(Level),
    SetRelease(Duration),
}

fn next_sequence_number() -> usize {
    static SEQUENCE_NUMBER: AtomicUsize = AtomicUsize::new(0);
    SEQUENCE_NUMBER.fetch_add(1, Ordering::Relaxed)
}

#[derive(PartialEq, PartialOrd)]
pub struct AdsrEvent {
    sequence_number: usize,
    time: Timestamp,
    event_type: AdsrEventType,
}

impl AdsrEvent {
    pub fn new(time: Timestamp, event_type: AdsrEventType) -> Self {
        Self {
            sequence_number: next_sequence_number(),
            time,
            event_type,
        }
    }

    pub fn get_event_type(&self) -> &AdsrEventType {
        &self.event_type
    }
}

impl EventProcessorEvent for AdsrEvent {
    fn get_time(&self) -> Timestamp {
        self.time
    }

    fn should_clear_queue(&self) -> bool {
        false
    }

    fn sequence_number(&self) -> usize {
        self.sequence_number
    }
}
