use crate::sources::realtime_oscillator::RealtimeOscillator;

use super::id::Id;

pub enum Command {
    Start,
    Stop,

    AddOscillator(RealtimeOscillator),
    RemoveOscillator(Id),
}
