use std::sync::Arc;

use crate::{
    graph::DspProcessor, AudioBuffer, BorrowedAudioBuffer, MutableBorrowedAudioBuffer,
    SampleLocation,
};
use itertools::izip;
use rustfft::{num_complex::Complex, num_traits::Zero, Fft, FftPlanner};

type ComplexAudioBuffer = Vec<Vec<Complex<f32>>>;

pub struct ConvolutionProcessor {
    fft: Arc<dyn Fft<f32>>,
    ifft: Arc<dyn Fft<f32>>,
    impulse_fft: ComplexAudioBuffer,
    input_fft: ComplexAudioBuffer,
    complex_input: ComplexAudioBuffer,
    complex_output: ComplexAudioBuffer,
    output_scale: f32,
    maximum_frame_count: usize,
}

impl ConvolutionProcessor {
    pub fn new(impulse: &dyn AudioBuffer, maximum_frame_count: usize) -> Self {
        let convolution_length =
            (impulse.frame_count() + maximum_frame_count - 1).next_power_of_two();
        let output_channel_count = impulse.channel_count();

        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(convolution_length);

        Self {
            fft: fft.clone(),
            ifft: planner.plan_fft_inverse(convolution_length),
            impulse_fft: fft_impulse(impulse, fft.as_ref(), convolution_length),
            input_fft: create_complex_audio_buffer(output_channel_count, convolution_length),
            complex_input: create_complex_audio_buffer(output_channel_count, convolution_length),
            complex_output: create_complex_audio_buffer(output_channel_count, convolution_length),
            output_scale: 1.0 / convolution_length as f32,
            maximum_frame_count,
        }
    }

    fn consume_input(&mut self, input: &dyn AudioBuffer) {
        debug_assert_eq!(input.channel_count(), self.complex_input.len());
        for channel in 0..input.channel_count() {
            let complex_input = self
                .complex_input
                .get_mut(channel)
                .expect("Invalid input channel count");

            complex_input.copy_within(input.frame_count().., 0);

            let destination_index = complex_input.len() - input.frame_count();

            let complex_input_slice = &mut complex_input[destination_index..];
            let audio_data = input.get_channel_data(SampleLocation::channel(channel));
            debug_assert_eq!(complex_input_slice.len(), audio_data.len());

            for (sample, complex_sample) in izip!(audio_data.iter(), complex_input_slice.iter_mut())
            {
                *complex_sample = Complex::new(*sample, 0.0_f32);
            }
        }
    }

    fn fft_input(&mut self) {
        for (complex_input, input_fft) in
            izip!(self.complex_input.iter(), self.input_fft.iter_mut())
        {
            input_fft.copy_from_slice(complex_input);
            self.fft.process(input_fft);
        }
    }

    fn perform_fft_multiplication(&mut self) {
        for (input_fft, impulse_fft, output_fft) in izip!(
            self.input_fft.iter(),
            self.impulse_fft.iter(),
            self.complex_output.iter_mut()
        ) {
            for (input_value, impulse_value, output_value) in
                izip!(input_fft, impulse_fft, output_fft)
            {
                *output_value = *input_value * *impulse_value;
            }
        }
    }

    fn ifft_output(&mut self) {
        for output_fft in self.complex_output.iter_mut() {
            self.ifft.process(output_fft);
        }
    }

    fn copy_to_output(&mut self, output: &mut dyn AudioBuffer) {
        debug_assert_eq!(output.channel_count(), self.complex_output.len());

        for channel in 0..output.channel_count() {
            let audio_data = output.get_channel_data_mut(SampleLocation::channel(channel));

            let convolution_output = self.complex_output.get(channel).expect("Invalid channel");

            let index = convolution_output.len() - audio_data.len();
            let convolution_output = &convolution_output[index..];

            for (output_sample, complex_convolution_output) in izip!(audio_data, convolution_output)
            {
                *output_sample = complex_convolution_output.re * self.output_scale;
            }
        }
    }

    fn process(&mut self, input: &dyn AudioBuffer, output: &mut dyn AudioBuffer) {
        debug_assert_eq!(input.channel_count(), output.channel_count());
        debug_assert_eq!(input.frame_count(), output.frame_count());

        let mut remaining = input.frame_count();
        let mut offset = 0;

        while remaining > 0 {
            let frames = std::cmp::min(remaining, self.maximum_frame_count);

            let input = BorrowedAudioBuffer::slice_frames(input, offset, frames);
            let mut output = MutableBorrowedAudioBuffer::slice_frames(output, offset, frames);

            self.consume_input(&input);
            self.fft_input();
            self.perform_fft_multiplication();
            self.ifft_output();
            self.copy_to_output(&mut output);

            remaining -= frames;
            offset += frames;
        }
    }
}

fn create_complex_audio_buffer(channel_count: usize, length: usize) -> ComplexAudioBuffer {
    (0..channel_count)
        .map(|_| vec![Complex::zero(); length])
        .collect()
}

fn fft_impulse(
    impulse: &dyn AudioBuffer,
    fft: &dyn Fft<f32>,
    convolution_length: usize,
) -> ComplexAudioBuffer {
    let mut impulse_fft = Vec::new();

    for channel in 0..impulse.channel_count() {
        let impulse_data = impulse.get_channel_data(SampleLocation::channel(channel));

        let mut impulse_data: Vec<Complex<f32>> = impulse_data
            .iter()
            .map(|sample| Complex::new(*sample, 0.0_f32))
            .collect();

        impulse_data.resize(convolution_length, Complex::zero());

        fft.process(&mut impulse_data);
        impulse_fft.push(impulse_data);
    }

    impulse_fft
}

impl DspProcessor for ConvolutionProcessor {
    fn process_audio(&mut self, context: &mut crate::ProcessContext) {
        self.process(context.input_buffer, context.output_buffer);
    }
}

#[cfg(test)]
mod tests {

    use std::iter::zip;

    use approx::assert_relative_eq;

    use crate::{
        AudioBuffer, BorrowedAudioBuffer, MutableBorrowedAudioBuffer, OwnedAudioBuffer,
        SampleLocation,
    };

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
                if impulse_frame <= output_frame && output_frame - impulse_frame < input_data.len()
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
    fn simple_convolution() {
        let signal_1 = [1.0, 2.0, 3.0, 4.0];
        let signal_2 = [5.0, 6.0, 7.0, 8.0];
        let expected_output = [5.0, 16.0, 34.0, 60.0, 61.0, 52.0, 32.0];

        let frame_count = expected_output.len();
        let channel_count = 1;
        let sample_rate = 44_100;

        let mut input_signal = OwnedAudioBuffer::from_slice(&signal_1, channel_count, sample_rate);
        input_signal = input_signal.padded_to_length(frame_count);

        let impulse_signal = OwnedAudioBuffer::from_slice(&signal_2, channel_count, sample_rate);

        let mut output_buffer = OwnedAudioBuffer::new(frame_count, channel_count, sample_rate);

        let mut processor = ConvolutionProcessor::new(&impulse_signal, expected_output.len());

        processor.process(&input_signal, &mut output_buffer);

        let output_data = output_buffer.get_channel_data(SampleLocation::origin());

        for (expected_sample, actual_sample) in izip!(expected_output.iter(), output_data.iter()) {
            assert_relative_eq!(expected_sample, actual_sample, epsilon = 1e-3);
        }
    }

    #[test]
    fn unit_impulse() {
        for impulse_length in [64, 1024, 4096] {
            println!("Impulse length = {impulse_length}");

            let frame_count = 1024;
            let channel_count = 1;
            let sample_rate = 48_000;

            let input = OwnedAudioBuffer::white_noise(frame_count, channel_count, sample_rate);

            let impulse = create_dirac(impulse_length, channel_count, sample_rate);

            let mut processor = ConvolutionProcessor::new(&impulse, frame_count);

            let mut processed =
                OwnedAudioBuffer::new(frame_count + impulse_length - 1, channel_count, sample_rate);

            let input = input.padded_to_length(processed.frame_count());

            processor.process(&input, &mut processed);

            let input_channel_data = input.get_channel_data(SampleLocation::origin());
            let processed_channel_data = processed.get_channel_data(SampleLocation::origin());

            for (input_sample, processed_sample) in
                zip(input_channel_data.iter(), processed_channel_data.iter())
            {
                assert_relative_eq!(input_sample, processed_sample, epsilon = 1e-3);
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

            let maximum_frame_count = 1024;
            let mut processor = ConvolutionProcessor::new(&impulse, maximum_frame_count);

            let naive_result = naive_convolution(&input, &impulse);

            let mut processed_result =
                OwnedAudioBuffer::new(naive_result.frame_count(), channel_count, sample_rate);

            let input = input.padded_to_length(naive_result.frame_count());

            processor.process(&input, &mut processed_result);

            let naive_data = naive_result.get_channel_data(SampleLocation::origin());
            let processed_data = processed_result.get_channel_data(SampleLocation::origin());

            for (naive_sample, processed_sample) in zip(naive_data.iter(), processed_data.iter()) {
                assert_relative_eq!(*naive_sample, *processed_sample, epsilon = 1e-3);
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

            let maximum_frame_count = 1024;
            let mut processor = ConvolutionProcessor::new(&impulse, maximum_frame_count);

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
                assert_relative_eq!(*naive_sample, *processed_sample, epsilon = 1e-3);
            }
        }
    }
}
