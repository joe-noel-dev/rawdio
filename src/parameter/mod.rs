mod audio_parameter;
mod parameter_change;
mod parameter_value;
mod realtime_parameter;

pub type AudioParameter = audio_parameter::AudioParameter;
pub(crate) type RealtimeAudioParameter = realtime_parameter::RealtimeAudioParameter;
pub(crate) type ParameterChange = parameter_change::ParameterChange;
