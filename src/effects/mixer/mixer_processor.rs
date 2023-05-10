use crate::{graph::DspProcessor, SampleLocation};

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
    fn process_audio(&mut self, context: &mut crate::ProcessContext) {
        self.process_events();

        for output_channel in 0..context.output_buffer.channel_count() {
            for input_channel in 0..context.input_buffer.channel_count() {
                let gain = self.gain_matrix.get_level(input_channel, output_channel);

                if gain.is_zero() {
                    continue;
                }

                let destination_location = SampleLocation::channel(output_channel);
                let source_location = SampleLocation::channel(input_channel);

                let frame_count = context.output_buffer.frame_count();
                let channel_count = 1;

                if gain.is_unity() {
                    context.output_buffer.add_from(
                        context.input_buffer,
                        source_location,
                        destination_location,
                        channel_count,
                        frame_count,
                    );
                } else {
                    context.output_buffer.add_from_with_gain(
                        context.input_buffer,
                        source_location,
                        destination_location,
                        channel_count,
                        frame_count,
                        gain.as_gain_f32(),
                    );
                }
            }
        }
    }
}
