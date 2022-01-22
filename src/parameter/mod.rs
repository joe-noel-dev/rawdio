use std::sync::Arc;

use atomic_float::AtomicF64;

use crate::Timestamp;

pub type ParameterValue = Arc<AtomicF64>;

#[derive(Clone, Copy, PartialEq, PartialOrd)]
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

pub(crate) mod audio_parameter;
pub(crate) mod realtime_parameter;
