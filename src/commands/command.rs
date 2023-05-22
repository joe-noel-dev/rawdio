use crate::graph::{Connection, Dsp, Endpoint};

use super::{id::Id, parameter_change_request::CancelChangeRequest, ParameterChangeRequest};

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
