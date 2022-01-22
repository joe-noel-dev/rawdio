use std::collections::HashMap;

use crate::{
    buffer::audio_buffer::AudioBuffer,
    commands::{
        command::{Command, ParameterChangeRequest},
        id::Id,
    },
    parameter::realtime_parameter::RealtimeAudioParameter,
    timestamp::Timestamp,
};

use lockfree::channel::mpsc::Sender;

pub type DspParameterMap = HashMap<Id, RealtimeAudioParameter>;

pub struct Dsp {
    id: Id,
    processor: Box<dyn DspProcessor + Send + Sync>,
    parameters: DspParameterMap,
}

pub trait DspProcessor {
    fn process_audio(
        &mut self,
        input_buffer: &dyn AudioBuffer,
        output_buffer: &mut dyn AudioBuffer,
        start_time: &Timestamp,
        parameters: &DspParameterMap,
    );
}

impl Dsp {
    pub fn new(
        id: Id,
        processor: Box<dyn DspProcessor + Send + Sync>,
        parameters: DspParameterMap,
    ) -> Self {
        Self {
            id,
            processor,
            parameters,
        }
    }

    pub fn add_to_audio_process(dsp: Self, command_queue: &Sender<Command>) {
        let _ = command_queue.send(Command::AddDsp(Box::new(dsp)));
    }

    pub fn remove_from_audio_process(id: Id, command_queue: &Sender<Command>) {
        let _ = command_queue.send(Command::RemoveDsp(id));
    }

    pub fn get_id(&self) -> Id {
        self.id
    }

    pub fn process_audio(
        &mut self,
        input_buffer: &dyn AudioBuffer,
        output_buffer: &mut dyn AudioBuffer,
        start_time: &Timestamp,
    ) {
        for (_, parameter) in self.parameters.iter_mut() {
            parameter.set_current_time(*start_time);
        }

        self.processor
            .process_audio(input_buffer, output_buffer, start_time, &self.parameters);
    }

    pub fn request_parameter_change(&mut self, parameter_change: ParameterChangeRequest) {
        if let Some(parameter) = self.parameters.get_mut(&parameter_change.parameter_id) {
            parameter.add_parameter_change(parameter_change.change)
        }
    }
}

#[cfg(test)]
mod tests {}
