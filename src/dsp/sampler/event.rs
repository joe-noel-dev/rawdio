use crate::Timestamp;

pub enum SampleEventType {
    Start(Timestamp),
    Stop,

    EnableLoop(Timestamp, Timestamp),
    CancelLoop,

    CancelAll,
}

pub struct SamplerEvent {
    pub time: Timestamp,
    pub event_type: SampleEventType,
}

impl SamplerEvent {
    pub fn start(start_at_time: Timestamp, position_in_sample: Timestamp) -> Self {
        Self {
            time: start_at_time,
            event_type: SampleEventType::Start(position_in_sample),
        }
    }

    pub fn start_now() -> Self {
        Self {
            time: Timestamp::zero(),
            event_type: SampleEventType::Start(Timestamp::zero()),
        }
    }

    pub fn stop(stop_at_time: Timestamp) -> Self {
        Self {
            time: stop_at_time,
            event_type: SampleEventType::Stop,
        }
    }

    pub fn stop_now() -> Self {
        Self {
            time: Timestamp::zero(),
            event_type: SampleEventType::Stop,
        }
    }

    pub fn enable_loop_at_time(enable_at_time: Timestamp, loop_start: Timestamp, loop_end: Timestamp) -> Self {
        Self {
            time: enable_at_time,
            event_type: SampleEventType::EnableLoop(loop_start, loop_end),
        }
    }

    pub fn cancel_loop_at_time(cancel_time: Timestamp) -> Self {
        Self {
            time: cancel_time,
            event_type: SampleEventType::CancelLoop,
        }
    }

    pub fn enable_loop(loop_start: Timestamp, loop_end: Timestamp) -> Self {
        Self {
            time: Timestamp::zero(),
            event_type: SampleEventType::EnableLoop(loop_start, loop_end),
        }
    }

    pub fn cancel_loop() -> Self {
        Self {
            time: Timestamp::zero(),
            event_type: SampleEventType::CancelLoop,
        }
    }

    pub fn cancel_all() -> Self {
        Self {
            time: Timestamp::zero(),
            event_type: SampleEventType::CancelAll,
        }
    }
}
