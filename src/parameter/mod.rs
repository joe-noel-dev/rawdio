use std::sync::Arc;

use atomic_float::AtomicF64;

use crate::Timestamp;

pub type ParameterValue = Arc<AtomicF64>;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd)]
pub enum ValueChangeMethod {
    Immediate,
    Linear,
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct ParameterChange {
    value: f64,
    end_time: Timestamp,
    method: ValueChangeMethod,
}

mod audio_parameter;
mod realtime_parameter;

pub type AudioParameter = audio_parameter::AudioParameter;
pub type RealtimeAudioParameter = realtime_parameter::RealtimeAudioParameter;
