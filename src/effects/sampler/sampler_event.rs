use std::sync::atomic::{AtomicUsize, Ordering};

use crate::{effects::utility::EventProcessorEvent, Timestamp};

#[derive(Debug, PartialEq, PartialOrd)]
pub enum SampleEventType {
    Start(Timestamp),
    StartImmediate,
    Stop,

    EnableLoop(Timestamp, Timestamp),
    CancelLoop,

    CancelAll,
}

fn next_sequence_number() -> usize {
    static SEQUENCE_NUMBER: AtomicUsize = AtomicUsize::new(0);
    SEQUENCE_NUMBER.fetch_add(1, Ordering::Relaxed)
}

#[derive(Debug)]
pub struct SamplerEvent {
    sequence_number: usize,
    time: Timestamp,
    event_type: SampleEventType,
}

impl SamplerEvent {
    pub fn get_event_type(&self) -> &SampleEventType {
        &self.event_type
    }

    pub fn start(start_at_time: Timestamp, position_in_sample: Timestamp) -> Self {
        Self {
            sequence_number: next_sequence_number(),
            time: start_at_time,
            event_type: SampleEventType::Start(position_in_sample),
        }
    }

    pub fn start_now() -> Self {
        Self {
            sequence_number: next_sequence_number(),
            time: Timestamp::zero(),
            event_type: SampleEventType::StartImmediate,
        }
    }

    pub fn stop(stop_at_time: Timestamp) -> Self {
        Self {
            sequence_number: next_sequence_number(),
            time: stop_at_time,
            event_type: SampleEventType::Stop,
        }
    }

    pub fn stop_now() -> Self {
        Self {
            sequence_number: next_sequence_number(),
            time: Timestamp::zero(),
            event_type: SampleEventType::Stop,
        }
    }

    pub fn enable_loop_at_time(
        enable_at_time: Timestamp,
        loop_start: Timestamp,
        loop_end: Timestamp,
    ) -> Self {
        Self {
            sequence_number: next_sequence_number(),
            time: enable_at_time,
            event_type: SampleEventType::EnableLoop(loop_start, loop_end),
        }
    }

    pub fn cancel_loop_at_time(cancel_time: Timestamp) -> Self {
        Self {
            sequence_number: next_sequence_number(),
            time: cancel_time,
            event_type: SampleEventType::CancelLoop,
        }
    }

    pub fn enable_loop(loop_start: Timestamp, loop_end: Timestamp) -> Self {
        Self {
            sequence_number: next_sequence_number(),
            time: Timestamp::zero(),
            event_type: SampleEventType::EnableLoop(loop_start, loop_end),
        }
    }

    pub fn cancel_loop() -> Self {
        Self {
            sequence_number: next_sequence_number(),
            time: Timestamp::zero(),
            event_type: SampleEventType::CancelLoop,
        }
    }

    pub fn cancel_all() -> Self {
        Self {
            sequence_number: next_sequence_number(),
            time: Timestamp::zero(),
            event_type: SampleEventType::CancelAll,
        }
    }
}

impl EventProcessorEvent for SamplerEvent {
    fn get_time(&self) -> Timestamp {
        self.time
    }

    fn should_clear_queue(&self) -> bool {
        self.event_type == SampleEventType::CancelAll
    }

    fn sequence_number(&self) -> usize {
        self.sequence_number
    }
}
