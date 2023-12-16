use super::{parameter_change_request::CancelChangeRequest, Id, ParameterChangeRequest};
use crate::graph::*;

pub enum Command {
    Start,
    Stop,

    AddDsp(Box<Dsp>),
    RemoveDsp(Id),

    CancelParameterChanges(CancelChangeRequest),
    ParameterValueChange(ParameterChangeRequest),

    AddConnection(Connection),
    RemoveConnection(Connection),
    ConnectToOutput(Endpoint),
    ConnectToInput(Endpoint),
}
