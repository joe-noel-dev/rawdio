use crate::graph::{Connection, Dsp, Endpoint};

use super::{id::Id, ParameterChangeRequest};

pub enum Command {
    Start,
    Stop,

    AddDsp(Box<Dsp>),
    RemoveDsp(Id),

    ParameterValueChange(ParameterChangeRequest),

    AddConnection(Connection),
    RemoveConnection(Connection),
    ConnectToOutput(Endpoint),
}
