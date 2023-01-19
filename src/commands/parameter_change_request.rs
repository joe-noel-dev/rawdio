use crate::{parameter::ParameterChange, Timestamp};

use super::Id;

pub struct ParameterChangeRequest {
    pub dsp_id: Id,
    pub parameter_id: Id,
    pub change: ParameterChange,
}

pub struct CancelChangeRequest {
    pub dsp_id: Id,
    pub parameter_id: Id,
    pub end_time: Option<Timestamp>,
}
