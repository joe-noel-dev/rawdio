use crate::{
    parameter::{ParameterChange, ParameterId},
    Timestamp,
};

use super::Id;

pub struct ParameterChangeRequest {
    pub dsp_id: Id,
    pub parameter_id: ParameterId,
    pub change: ParameterChange,
}

pub struct CancelChangeRequest {
    pub dsp_id: Id,
    pub parameter_id: ParameterId,
    pub end_time: Option<Timestamp>,
}
