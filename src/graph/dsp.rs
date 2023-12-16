use crate::{
    commands::{CancelChangeRequest, Command, Id, ParameterChangeRequest},
    engine::CommandQueue,
    prelude::*,
};

use super::DspParameters;

pub struct Dsp {
    id: Id,
    input_count: usize,
    output_count: usize,
    processor: Box<dyn DspProcessor + Send + Sync>,
    parameters: DspParameters,
}

/// The context that is passed to every [DspProcessor]
pub struct ProcessContext<'a> {
    /// The input buffer, containing the input channels into this node
    pub input_buffer: &'a dyn AudioBuffer,

    /// The output buffer, to be filled by this node
    pub output_buffer: &'a mut dyn AudioBuffer,

    /// The start time of the process block
    pub start_time: &'a Timestamp,

    /// The audio-rate parameters
    pub parameters: &'a DspParameters,
}

pub trait DspProcessor {
    fn process_audio(&mut self, context: &mut ProcessContext);
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

    pub fn add_to_audio_process(self, command_queue: &dyn CommandQueue) {
        command_queue.send(Command::AddDsp(Box::new(self)));
    }

    pub fn remove_from_audio_process(id: Id, command_queue: &dyn CommandQueue) {
        command_queue.send(Command::RemoveDsp(id));
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
        assert_eq!(input_buffer.channel_count(), self.input_count);
        assert_eq!(output_buffer.channel_count(), self.output_count);

        for (_, parameter) in self.parameters.iter_mut() {
            parameter.process(
                start_time,
                output_buffer.frame_count(),
                output_buffer.sample_rate(),
            );
        }

        self.processor.process_audio(&mut ProcessContext {
            input_buffer,
            output_buffer,
            start_time,
            parameters: &self.parameters,
        });
    }

    pub fn request_parameter_change(&mut self, parameter_change: ParameterChangeRequest) {
        let parameter = self
            .parameters
            .get_parameter_mut(parameter_change.parameter_id);
        parameter.add_parameter_change(parameter_change.change);
    }

    pub fn cancel_parameter_changes(&mut self, change_request: CancelChangeRequest) {
        let parameter = self
            .parameters
            .get_parameter_mut(change_request.parameter_id);

        match change_request.end_time {
            Some(end_time) => parameter.cancel_scheduled_changes_ending_after(&end_time),
            None => parameter.cancel_scheduled_changes(),
        }
    }
}

#[cfg(test)]
mod tests {}
