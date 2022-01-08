use std::collections::HashMap;

use crate::{
    commands::{command::ParameterChangeRequest, id::Id},
    parameter::RealtimeAudioParameter,
    timestamp::Timestamp,
    utility::audio_buffer::AudioBuffer,
};

pub type DspParameterMap = HashMap<Id, RealtimeAudioParameter>;
pub type DspProcessFn =
    Box<dyn FnMut(&mut dyn AudioBuffer, &Timestamp, &DspParameterMap) + Send + Sync>;

pub struct Dsp {
    id: Id,
    process: DspProcessFn,
    parameters: DspParameterMap,
}

impl Dsp {
    pub fn new(id: Id, process: DspProcessFn, parameters: DspParameterMap) -> Self {
        Self {
            id,
            process,
            parameters,
        }
    }

    pub fn get_id(&self) -> Id {
        self.id
    }

    pub fn process_audio(&mut self, output_buffer: &mut dyn AudioBuffer, start_time: &Timestamp) {
        for (_, parameter) in self.parameters.iter_mut() {
            parameter.set_current_time(*start_time);
        }

        (self.process)(output_buffer, start_time, &self.parameters);
    }

    pub fn request_parameter_change(&mut self, parameter_change: ParameterChangeRequest) {
        if let Some(parameter) = self.parameters.get_mut(&parameter_change.parameter_id) {
            parameter.add_parameter_change(parameter_change.change)
        }
    }
}
