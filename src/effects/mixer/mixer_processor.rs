use crate::{
    graph::{DspParameters, DspProcessor},
    AudioBuffer, SampleLocation, Timestamp,
};

use super::{mixer_event::EventReceiver, mixer_matrix::MixerMatrix};

pub struct MixerProcessor {
    event_receiver: EventReceiver,
    gain_matrix: MixerMatrix,
}

impl MixerProcessor {
    pub fn new(event_receiver: EventReceiver) -> Self {
        Self {
            event_receiver,
            gain_matrix: MixerMatrix::empty(),
        }
    }

    fn process_events(&mut self) {
        while let Ok(matrix) = self.event_receiver.try_recv() {
            self.gain_matrix = matrix;
        }
    }
}

impl DspProcessor for MixerProcessor {
    fn process_audio(
        &mut self,
        input_buffer: &dyn AudioBuffer,
        output_buffer: &mut dyn AudioBuffer,
        _start_time: &Timestamp,
        _parameters: &DspParameters,
    ) {
        self.process_events();

        for output_channel in 0..output_buffer.channel_count() {
            for input_channel in 0..input_buffer.channel_count() {
                let gain = self.gain_matrix.get_level(input_channel, output_channel);

                let output_location = SampleLocation::origin().with_channel(output_channel);
                let input_location = SampleLocation::origin().with_channel(input_channel);

                let output = output_buffer.get_channel_data_mut(output_location);
                let input = input_buffer.get_channel_data(input_location);

                for (output_sample, input_sample) in output.iter_mut().zip(input.iter()) {
                    *output_sample = *input_sample * gain.as_gain() as f32;
                }
            }
        }
    }
}
