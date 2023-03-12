use crate::{AudioBuffer, OwnedAudioBuffer, SampleLocation};

struct ConvolutionProcessor {
    impulse: OwnedAudioBuffer,
    overflow: OwnedAudioBuffer,
    overflow_in_use: usize,
}

impl ConvolutionProcessor {
    fn new(impulse: OwnedAudioBuffer) -> Self {
        let overflow = OwnedAudioBuffer::new(
            impulse.frame_count(),
            impulse.channel_count(),
            impulse.sample_rate(),
        );

        Self {
            impulse,
            overflow,
            overflow_in_use: 0,
        }
    }

    fn process(&mut self, input: &dyn AudioBuffer, output: &mut dyn AudioBuffer) {
        debug_assert!(output.channel_count() <= self.impulse.channel_count());
        debug_assert!(output.channel_count() <= self.overflow.channel_count());

        let frames_to_copy = std::cmp::min(self.overflow_in_use, output.frame_count());
        self.overflow_in_use -= frames_to_copy;

        for channel in 0..output.channel_count() {
            output.copy_from(
                &self.overflow,
                SampleLocation::origin(),
                SampleLocation::origin(),
                output.channel_count(),
                frames_to_copy,
            );

            self.overflow.copy_within(
                channel,
                frames_to_copy,
                0,
                self.overflow.frame_count() - frames_to_copy,
            );

            self.overflow.clear_range(
                channel,
                self.overflow.frame_count() - frames_to_copy,
                frames_to_copy,
            );
        }

        let result_length = input.frame_count() + self.impulse.frame_count();

        for channel in 0..output.channel_count() {
            let output_data = output.get_channel_data_mut(SampleLocation::channel(channel));

            let overflow_data = self
                .overflow
                .get_channel_data_mut(SampleLocation::channel(channel));

            let impulse_data = self
                .impulse
                .get_channel_data(SampleLocation::channel(channel));

            let input_data = input.get_channel_data(SampleLocation::channel(channel));

            for result_frame in 0..result_length {
                let output_sample = if result_frame < output_data.len() {
                    &mut output_data[result_frame]
                } else {
                    let index = result_frame - output_data.len();
                    &mut overflow_data[index]
                };

                for (inpulse_frame, inpulse_sample) in impulse_data.iter().enumerate() {
                    if result_frame >= inpulse_frame
                        && result_frame - inpulse_frame < input_data.len()
                    {
                        *output_sample +=
                            input_data[result_frame - inpulse_frame] * *inpulse_sample;
                    }
                }
            }
        }

        self.overflow_in_use = if result_length > output.frame_count() {
            result_length - output.frame_count()
        } else {
            0
        };
    }
}

#[cfg(test)]
mod tests {

    use std::iter::zip;

    use approx::assert_relative_eq;

    use crate::{AudioBuffer, BorrowedAudioBuffer, MutableBorrowedAudioBuffer, SampleLocation};

    use super::*;

    fn naive_convolution(input: &dyn AudioBuffer, impulse: &dyn AudioBuffer) -> OwnedAudioBuffer {
        debug_assert_eq!(input.sample_rate(), impulse.sample_rate());
        debug_assert_eq!(input.channel_count(), impulse.channel_count());

        let result_length = input.frame_count() + impulse.frame_count() - 1;

        let mut result =
            OwnedAudioBuffer::new(result_length, input.channel_count(), input.sample_rate());

        let output_data = result.get_channel_data_mut(SampleLocation::origin());
        let impulse_data = impulse.get_channel_data(SampleLocation::origin());
        let input_data = input.get_channel_data(SampleLocation::origin());

        for (output_frame, output_sample) in output_data.iter_mut().enumerate() {
            for (impulse_frame, inpulse_sample) in impulse_data.iter().enumerate() {
                if output_frame >= impulse_frame && output_frame - impulse_frame < input_data.len()
                {
                    *output_sample += input_data[output_frame - impulse_frame] * *inpulse_sample;
                }
            }
        }

        result
    }

    fn create_dirac(length: usize, channel_count: usize, sample_rate: usize) -> OwnedAudioBuffer {
        let mut impulse = OwnedAudioBuffer::new(length, channel_count, sample_rate);

        impulse.clear();

        let impulse_data = impulse.get_channel_data_mut(SampleLocation::origin());
        impulse_data[0] = 1.0;

        impulse
    }

    #[test]
    fn unit_impulse() {
        for impulse_length in [1, 2, 64, 1024, 4096] {
            println!("Impulse length = {impulse_length}");

            let frame_count = 1024;
            let channel_count = 1;
            let sample_rate = 48_000;

            let input = OwnedAudioBuffer::white_noise(frame_count, channel_count, sample_rate);

            let impulse = create_dirac(impulse_length, channel_count, sample_rate);

            let mut processor = ConvolutionProcessor::new(impulse);

            let mut processed =
                OwnedAudioBuffer::new(frame_count + impulse_length - 1, channel_count, sample_rate);

            processor.process(&input, &mut processed);

            let input_channel_data = input.get_channel_data(SampleLocation::origin());
            let processed_channel_data = processed.get_channel_data(SampleLocation::origin());

            for (input_sample, processed_sample) in
                zip(input_channel_data.iter(), processed_channel_data.iter())
            {
                assert_relative_eq!(input_sample, processed_sample, epsilon = 1e-6);
            }
        }
    }

    #[test]
    fn generates_correct_output() {
        for (input_length, impulse_length) in [(1024, 1024), (1024, 8192), (8192, 1024)] {
            let channel_count = 1;
            let sample_rate = 48_000;

            let impulse = OwnedAudioBuffer::white_noise(impulse_length, channel_count, sample_rate);

            let input = OwnedAudioBuffer::white_noise(input_length, channel_count, sample_rate);

            let mut processor = ConvolutionProcessor::new(impulse.clone());

            let naive_result = naive_convolution(&input, &impulse);

            let mut processed_result =
                OwnedAudioBuffer::new(naive_result.frame_count(), channel_count, sample_rate);

            processor.process(&input, &mut processed_result);

            let naive_data = naive_result.get_channel_data(SampleLocation::origin());
            let processed_data = processed_result.get_channel_data(SampleLocation::origin());

            for (naive_sample, processed_sample) in zip(naive_data.iter(), processed_data.iter()) {
                assert_relative_eq!(*naive_sample, *processed_sample, epsilon = 1e-6);
            }
        }
    }

    #[test]
    fn process_in_chunks() {
        for (input_length, impulse_length) in [(1024, 1024), (1024, 8192), (8192, 1024)] {
            let channel_count = 1;
            let sample_rate = 48_000;

            let impulse = OwnedAudioBuffer::white_noise(impulse_length, channel_count, sample_rate);

            let input = OwnedAudioBuffer::white_noise(input_length, channel_count, sample_rate);

            let mut processor = ConvolutionProcessor::new(impulse.clone());

            let naive_result = naive_convolution(&input, &impulse);

            let mut padded_input =
                OwnedAudioBuffer::new(naive_result.frame_count(), channel_count, sample_rate);

            padded_input.copy_from(
                &input,
                SampleLocation::origin(),
                SampleLocation::origin(),
                input.channel_count(),
                input.frame_count(),
            );

            let step = 512;

            let mut processed_result =
                OwnedAudioBuffer::new(naive_result.frame_count(), channel_count, sample_rate);

            for offset in (0..naive_result.frame_count()).step_by(step) {
                let this_time = std::cmp::min(step, naive_result.frame_count() - offset);

                let input_slice =
                    BorrowedAudioBuffer::slice_frames(&padded_input, offset, this_time);

                let mut output_slice = MutableBorrowedAudioBuffer::slice_frames(
                    &mut processed_result,
                    offset,
                    this_time,
                );

                processor.process(&input_slice, &mut output_slice);
            }

            let naive_data = naive_result.get_channel_data(SampleLocation::origin());
            let processed_data = processed_result.get_channel_data(SampleLocation::origin());

            for (naive_sample, processed_sample) in zip(naive_data.iter(), processed_data.iter()) {
                assert_relative_eq!(*naive_sample, *processed_sample, epsilon = 1e-4);
            }
        }
    }
}
