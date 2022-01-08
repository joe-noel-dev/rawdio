use crate::{graph::dsp::Dsp, parameter::ParameterChange};

use super::id::Id;

pub struct ParameterChangeRequest {
    pub dsp_id: Id,
    pub parameter_id: Id,
    pub change: ParameterChange,
}

pub enum Command {
    Start,
    Stop,

    AddDsp(Box<Dsp>),
    RemoveDsp(Id),

    ParameterValueChange(ParameterChangeRequest),
}
