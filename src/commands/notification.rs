use crate::{sources::realtime_oscillator::RealtimeOscillator, timestamp};

pub enum Notification {
    Position(timestamp::Timestamp),
    DisposeOscillator(RealtimeOscillator),
}
