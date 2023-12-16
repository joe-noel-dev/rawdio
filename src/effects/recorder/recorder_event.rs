use crate::{
    effects::{utility::EventProcessorEvent, Channel},
    prelude::*,
};
use std::sync::atomic::{AtomicUsize, Ordering};

pub enum RecorderEventType {
    ReturnBuffer(OwnedAudioBuffer),
    Stop,
    Start,
}

pub struct RecorderEvent {
    pub sequence_number: usize,
    pub time: Timestamp,
    pub event_type: RecorderEventType,
}

pub type RecorderEventTransmitter = Channel::Sender<RecorderEvent>;
pub type RecorderEventReceiver = Channel::Receiver<RecorderEvent>;

fn next_sequence_number() -> usize {
    static SEQUENCE_NUMBER: AtomicUsize = AtomicUsize::new(0);
    SEQUENCE_NUMBER.fetch_add(1, Ordering::Relaxed)
}

impl RecorderEvent {
    pub fn start_now() -> Self {
        Self {
            sequence_number: next_sequence_number(),
            time: Timestamp::zero(),
            event_type: RecorderEventType::Start,
        }
    }

    pub fn stop_now() -> Self {
        Self {
            sequence_number: next_sequence_number(),
            time: Timestamp::zero(),
            event_type: RecorderEventType::Stop,
        }
    }

    pub fn stop_at_time(time: Timestamp) -> Self {
        Self {
            sequence_number: next_sequence_number(),
            time,
            event_type: RecorderEventType::Stop,
        }
    }

    pub fn return_buffer(buffer: OwnedAudioBuffer) -> Self {
        Self {
            sequence_number: next_sequence_number(),
            time: Timestamp::zero(),
            event_type: RecorderEventType::ReturnBuffer(buffer),
        }
    }
}

impl EventProcessorEvent for RecorderEvent {
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
