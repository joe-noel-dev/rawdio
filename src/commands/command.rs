use crate::graph::dsp::Dsp;

use super::{id::Id, parameter_command::ParameterCommand};

pub enum Command {
    Start,
    Stop,

    AddDsp(Dsp),
    RemoveDsp(Id),

    ParameterCommand(ParameterCommand),
}
