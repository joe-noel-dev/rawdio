use crate::{graph::dsp::Dsp, timestamp::Timestamp};

use super::id::Id;
use crate::parameter::RealtimeAudioParameter;

pub enum Command {
    Start,
    Stop,

    AddDsp(Box<Dsp>),
    RemoveDsp(Id),

    AddParameter(Box<RealtimeAudioParameter>),
    RemoveParameter(Id),

    SetValueImmediate((Id, f64)),
    LinearRampToValue((Id, f64, Timestamp)),
}
