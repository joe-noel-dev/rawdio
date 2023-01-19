use crate::parameter::ParameterChange;

use super::Id;

pub struct ParameterChangeRequest {
    pub dsp_id: Id,
    pub parameter_id: Id,
    pub change: ParameterChange,
}
