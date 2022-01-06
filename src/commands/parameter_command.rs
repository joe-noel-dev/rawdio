use crate::parameter::{ParameterValue, RealtimeAudioParameter};

use super::id::Id;

pub enum ParameterCommand {
    Add(RealtimeAudioParameter),
    SetValueImmediate((Id, ParameterValue)),
}
