use crate::graph::dsp::Dsp;

use super::id::Id;
use crate::parameter::RealtimeAudioParameter;

pub enum Command {
    Start,
    Stop,

    AddDsp(Box<Dsp>),
    RemoveDsp(Id),

    AddParameter(Box<RealtimeAudioParameter>),

    SetValueImmediate((Id, f32)),
}
