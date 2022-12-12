use crate::{
    commands::{command::ParameterChangeRequest, Command, Id},
    timestamp::Timestamp,
    AudioBuffer, CommandQueue,
};

use super::DspParameters;

pub struct Dsp {
    id: Id,
    input_count: usize,
    output_count: usize,
    processor: Box<dyn DspProcessor + Send + Sync>,
    parameters: DspParameters,
}

pub trait DspProcessor {
    fn process_audio(
        &mut self,
        input_buffer: &dyn AudioBuffer,
        output_buffer: &mut dyn AudioBuffer,
        start_time: &Timestamp,
        parameters: &DspParameters,
    );
}

impl Dsp {
    pub fn new(
        id: Id,
        input_count: usize,
        output_count: usize,
        processor: Box<dyn DspProcessor + Send + Sync>,
        parameters: DspParameters,
    ) -> Self {
        Self {
            id,
            input_count,
            output_count,
            processor,
            parameters,
        }
    }

    pub fn input_count(&self) -> usize {
        self.input_count
    }

    pub fn output_count(&self) -> usize {
        self.output_count
    }

    pub fn add_to_audio_process(self, command_queue: &CommandQueue) {
        let _ = command_queue.send(Command::AddDsp(Box::new(self)));
    }

    pub fn remove_from_audio_process(id: Id, command_queue: &CommandQueue) {
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
        assert_eq!(input_buffer.num_channels(), self.input_count);
        assert_eq!(output_buffer.num_channels(), self.output_count);

        for (_, parameter) in self.parameters.iter_mut() {
            parameter.prepare_value_buffer(
                start_time,
                output_buffer.num_frames(),
                output_buffer.sample_rate(),
            );
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
