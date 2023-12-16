use std::sync::Arc;

use crate::{
    dsp::mix_into_with_gains, graph::DspProcessor, AudioBuffer, BorrowedAudioBuffer,
    MutableBorrowedAudioBuffer, SampleLocation,
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

    fn copy_dry_to_output(input: &dyn AudioBuffer, output: &mut dyn AudioBuffer, dry: &[f32]) {
        debug_assert_eq!(input.channel_count(), output.channel_count());
        debug_assert_eq!(input.frame_count(), output.frame_count());
        debug_assert_eq!(input.frame_count(), dry.len());

        for channel in 0..output.channel_count() {
            let output = output.get_channel_data_mut(SampleLocation::channel(channel));
            let input = input.get_channel_data(SampleLocation::channel(channel));
            mix_into_with_gains(input, output, dry);
        }
    }

    fn copy_processed_to_output(
        complex_output: &ComplexAudioBuffer,
        output_scale: f32,
        output: &mut dyn AudioBuffer,
        wet: &[f32],
    ) {
        debug_assert_eq!(output.channel_count(), complex_output.len());
        debug_assert!(wet.len() >= output.frame_count());

        for channel in 0..output.channel_count() {
            let audio_data = output.get_channel_data_mut(SampleLocation::channel(channel));

            let convolution_output = complex_output.get(channel).expect("Invalid channel");

            let index = convolution_output.len() - audio_data.len();
            let convolution_output = &convolution_output[index..];

            for (output_sample, complex_convolution_output, wet) in
                izip!(audio_data, convolution_output, wet)
            {
                *output_sample += complex_convolution_output.re * output_scale * *wet;
            }
        }
    }

    fn process(
        &mut self,
        input: &dyn AudioBuffer,
        output: &mut dyn AudioBuffer,
        wet: &[f32],
        dry: &[f32],
    ) {
        debug_assert_eq!(input.channel_count(), output.channel_count());
        debug_assert_eq!(input.frame_count(), output.frame_count());
        debug_assert_eq!(input.frame_count(), wet.len());
        debug_assert_eq!(input.frame_count(), dry.len());

        for offset in (0..input.frame_count()).step_by(self.maximum_frame_count) {
            let frame_count = std::cmp::min(self.maximum_frame_count, input.frame_count() - offset);

            let wet = &wet[offset..offset + frame_count];
            let dry = &dry[offset..offset + frame_count];
            let input = BorrowedAudioBuffer::slice_frames(input, offset, frame_count);
            let mut output = MutableBorrowedAudioBuffer::slice_frames(output, offset, frame_count);

            self.consume_input(&input);
            self.fft_input();
            self.perform_fft_multiplication();
            self.ifft_output();

            Self::copy_dry_to_output(&input, &mut output, dry);
            Self::copy_processed_to_output(
                &self.complex_output,
                self.output_scale,
                &mut output,
                wet,
            );
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
        let wet = context
            .parameters
            .get_parameter_values("wet", context.output_buffer.frame_count());

        let dry = context
            .parameters
            .get_parameter_values("dry", context.output_buffer.frame_count());

        self.process(context.input_buffer, context.output_buffer, wet, dry);
    }
}

#[cfg(test)]
mod tests {

    use rand::Rng;
    use std::iter::zip;

    use approx::assert_relative_eq;

    use crate::{AudioBuffer, OwnedAudioBuffer, SampleLocation};

    use super::*;

    fn naive_convolution(input: &[f32], impulse: &[f32]) -> Vec<f32> {
        let result_length = input.len() + impulse.len() - 1;

        let mut result = vec![0.0; result_length];

        for (output_frame, output_sample) in result.iter_mut().enumerate() {
            for (impulse_frame, inpulse_sample) in impulse.iter().enumerate() {
                if impulse_frame <= output_frame && output_frame - impulse_frame < input.len() {
                    *output_sample += input[output_frame - impulse_frame] * *inpulse_sample;
                }
            }
        }

        result
    }

    fn dirac(length: usize) -> Vec<f32> {
        let mut impulse = vec![0.0; length];
        impulse[0] = 1.0;
        impulse
    }

    fn random_signal(length: usize) -> Vec<f32> {
        let mut rng = rand::thread_rng();
        (0..length).map(|_| rng.gen_range(-1.0..=1.0)).collect()
    }

    struct Fixture {
        processor: ConvolutionProcessor,
        sample_rate: usize,
    }

    impl Fixture {
        fn new(impulse: &[f32], maximum_frame_count: usize) -> Self {
            let channel_count = 1;
            let sample_rate = 48_000;
            let impulse = OwnedAudioBuffer::from_slice(impulse, channel_count, sample_rate);

            Self {
                processor: ConvolutionProcessor::new(&impulse, maximum_frame_count),
                sample_rate,
            }
        }

        fn process(&mut self, input: &[f32], wet: f32, dry: f32) -> Vec<f32> {
            let channel_count = 1;

            let mut output = OwnedAudioBuffer::new(input.len(), channel_count, self.sample_rate);

            let input = OwnedAudioBuffer::from_slice(input, channel_count, self.sample_rate);

            let wet = vec![wet; input.frame_count()];
            let dry = vec![dry; input.frame_count()];

            self.processor.process(&input, &mut output, &wet, &dry);

            output.get_channel_data(SampleLocation::origin()).to_vec()
        }
    }

    #[test]
    fn simple_convolution() {
        let signal_1 = [1.0, 2.0, 3.0, 4.0];
        let signal_2 = [5.0, 6.0, 7.0, 8.0];
        let expected_output = [5.0, 16.0, 34.0, 60.0, 61.0, 52.0, 32.0];

        let frame_count = expected_output.len();

        let mut fixture = Fixture::new(&signal_1, frame_count);
        let output = fixture.process(&signal_2, 1.0, 0.0);

        for (expected_sample, actual_sample) in izip!(expected_output.iter(), output.iter()) {
            assert_relative_eq!(expected_sample, actual_sample, epsilon = 1e-3);
        }
    }

    #[test]
    fn unit_impulse() {
        for impulse_length in [64, 1024, 4096] {
            println!("Impulse length = {impulse_length}");

            let frame_count = 1024;
            let expected_length = frame_count + impulse_length - 1;

            let mut input = random_signal(frame_count);
            input.resize(expected_length, 0.0);

            let impulse = dirac(impulse_length);

            let mut fixture = Fixture::new(&impulse, frame_count);

            let output = fixture.process(&input, 1.0, 0.0);

            for (input_sample, processed_sample) in zip(input.iter(), output.iter()) {
                assert_relative_eq!(input_sample, processed_sample, epsilon = 1e-3);
            }
        }
    }

    #[test]
    fn generates_correct_output() {
        for (input_length, impulse_length) in [(1024, 1024), (1024, 8192), (8192, 1024)] {
            let impulse = random_signal(impulse_length);
            let mut input = random_signal(input_length);

            let maximum_frame_count = 1024;

            let mut fixture = Fixture::new(&impulse, maximum_frame_count);

            let naive_result = naive_convolution(&input, &impulse);

            input.resize(naive_result.len(), 0.0);
            let processed = fixture.process(&input, 1.0, 0.0);

            for (naive_sample, processed_sample) in zip(naive_result.iter(), processed.iter()) {
                assert_relative_eq!(*naive_sample, *processed_sample, epsilon = 1e-3);
            }
        }
    }

    #[test]
    fn process_in_chunks() {
        for (input_length, impulse_length) in [(1024, 1024), (1024, 8192), (8192, 1024)] {
            let impulse = random_signal(impulse_length);
            let mut input = random_signal(input_length);

            let maximum_frame_count = 1024;

            let mut fixture = Fixture::new(&impulse, maximum_frame_count);

            let naive = naive_convolution(&input, &impulse);
            input.resize(naive.len(), 0.0);

            let mut result = vec![];

            let step = 512;

            for offset in (0..naive.len()).step_by(step) {
                let frames = std::cmp::min(step, naive.len() - offset);

                let input = &input[offset..offset + frames];
                let processed = fixture.process(input, 1.0, 0.0);
                result.extend(processed);
            }

            assert_eq!(result.len(), naive.len());

            for (naive_sample, processed_sample) in zip(naive.iter(), result.iter()) {
                assert_relative_eq!(*naive_sample, *processed_sample, epsilon = 1e-3);
            }
        }
    }

    #[test]
    fn wet_dry() {
        let impulse_length = 1024;
        let input_length = 2048;

        let impulse = random_signal(impulse_length);
        let mut input = random_signal(input_length);

        let maximum_frame_count = 1024;

        let mut fixture = Fixture::new(&impulse, maximum_frame_count);

        let naive = naive_convolution(&input, &impulse);
        input.resize(naive.len(), 0.0);

        let wet = 0.25;
        let dry = 1.0 - wet;
        let processed = fixture.process(&input, wet, dry);

        for (input_sample, naive_sample, processed_sample) in
            izip!(input.iter(), naive.iter(), processed.iter())
        {
            let expected = *naive_sample * wet + *input_sample * dry;
            assert_relative_eq!(expected, *processed_sample, epsilon = 1e-3);
        }
    }
}
