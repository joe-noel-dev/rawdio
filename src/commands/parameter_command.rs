use crate::parameter::RealtimeAudioParameter;

use super::id::Id;

pub enum ParameterCommand {
    Add(RealtimeAudioParameter),
    SetValueImmediate((Id, f32)),
}
