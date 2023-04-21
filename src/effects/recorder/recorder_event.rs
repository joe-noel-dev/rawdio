use crate::{
    effects::{utility::EventProcessorEvent, Channel},
    OwnedAudioBuffer, Timestamp,
};

pub enum RecorderEventType {
    Start,
    ReturnBuffer(OwnedAudioBuffer),
    Stop,
}

pub struct RecorderEvent {
    pub time: Timestamp,
    pub event_type: RecorderEventType,
}

pub type RecorderEventTransmitter = Channel::Sender<RecorderEvent>;
pub type RecorderEventReceiver = Channel::Receiver<RecorderEvent>;

impl RecorderEvent {
    pub fn start_now() -> Self {
        Self {
            time: Timestamp::zero(),
            event_type: RecorderEventType::Start,
        }
    }

    pub fn stop_now() -> Self {
        Self {
            time: Timestamp::zero(),
            event_type: RecorderEventType::Stop,
        }
    }

    pub fn stop_at_time(time: Timestamp) -> Self {
        Self {
            time,
            event_type: RecorderEventType::Stop,
        }
    }

    pub fn return_buffer(buffer: OwnedAudioBuffer) -> Self {
        Self {
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
}
