use itertools::izip;

use crate::{graph::DspProcessor, prelude::*};

pub struct OscillatorProcessor {
    phase: f64,
    wavetable: Vec<f64>,
}

impl OscillatorProcessor {
    pub fn new(wavetable: Vec<f64>) -> Self {
        Self {
            phase: 0.0,
            wavetable,
        }
    }

    fn increment_phase(&mut self, frequency: f32, sample_rate: usize) {
        self.phase += (frequency as f64) / (sample_rate as f64);
        while self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        debug_assert!(0.0 <= self.phase && self.phase < 1.0);
    }

    fn get_value(&self) -> f64 {
        let offset = self.phase * self.wavetable.len() as f64;

        let offset_before = offset.floor() as usize;
        let offset_after = offset.ceil() as usize;

        debug_assert!(offset_before < self.wavetable.len());
        debug_assert!(offset_after <= self.wavetable.len());

        let value_before = self.wavetable[offset_before];
        let value_after = if offset_after < self.wavetable.len() {
            self.wavetable[offset_after]
        } else {
            self.wavetable[0]
        };

        let weighting = offset - offset.floor();
        interpolate(value_before, value_after, weighting)
    }
}

fn interpolate(a: f64, b: f64, amount_of_b: f64) -> f64 {
    (1.0 - amount_of_b) * a + amount_of_b * b
}

impl DspProcessor for OscillatorProcessor {
    fn process_audio(&mut self, context: &mut crate::ProcessContext) {
        let sample_rate = context.output_buffer.sample_rate();

        let frequency_values = context
            .parameters
            .get_parameter_values("frequency", context.output_buffer.frame_count());
        let gain_values = context
            .parameters
            .get_parameter_values("gain", context.output_buffer.frame_count());

        let channel_count = context.output_buffer.channel_count();

        let location = SampleLocation::channel(0);
        let channel_data = context.output_buffer.get_channel_data_mut(location);

        for (sample, frequency) in izip!(channel_data.iter_mut(), frequency_values.iter()) {
            *sample = self.get_value() as f32;
            self.increment_phase(*frequency, sample_rate);
        }

        context.output_buffer.apply_gain(gain_values);

        (1..channel_count).for_each(|channel| {
            let location = SampleLocation::channel(0);
            context.output_buffer.duplicate_channel(
                location,
                channel,
                context.output_buffer.frame_count(),
            );
        });
    }
}

#[cfg(test)]
mod tests {

    use std::{
        sync::{atomic::Ordering, Arc},
        time::Duration,
    };

    use atomic_float::AtomicF64;

    use crate::{graph::DspParameters, parameter::RealtimeAudioParameter, ProcessContext};

    use super::*;

    fn process(
        processor: &mut impl DspProcessor,
        duration: Duration,
        sample_rate: usize,
        frame_count: usize,
        parameters: &mut DspParameters,
    ) -> Vec<f32> {
        let total_frame_count = (sample_rate as f64 * duration.as_secs_f64()).ceil() as usize;
        let channel_count = 1;
        let input_buffer = OwnedAudioBuffer::new(total_frame_count, channel_count, sample_rate);
        let mut output_buffer =
            OwnedAudioBuffer::new(total_frame_count, channel_count, sample_rate);

        let mut remaining = total_frame_count;

        while remaining > 0 {
            let offset = total_frame_count - remaining;

            let current_frame_count = remaining.min(frame_count);

            let input_buffer =
                BorrowedAudioBuffer::slice_frames(&input_buffer, offset, current_frame_count);

            let mut output_buffer = MutableBorrowedAudioBuffer::slice_frames(
                &mut output_buffer,
                offset,
                current_frame_count,
            );

            let start_time = Timestamp::from_samples(offset as f64, sample_rate);

            parameters.iter_mut().for_each(|(_, param)| {
                param.process(&start_time, current_frame_count, sample_rate);
            });

            let mut context = ProcessContext {
                input_buffer: &input_buffer,
                output_buffer: &mut output_buffer,
                start_time: &start_time,
                parameters,
            };

            processor.process_audio(&mut context);

            remaining -= current_frame_count;
        }

        output_buffer
            .get_channel_data(SampleLocation::origin())
            .to_vec()
    }

    #[test]
    fn test_oscillator() {
        let wavetable = vec![0.0, 0.25, 0.5, 0.75, 1.0, 0.75, 0.5, 0.25];
        let mut processor = OscillatorProcessor::new(wavetable);

        let frame_count = 1024;
        let sample_rate = 48_000;

        let frequency = Arc::new(AtomicF64::new(100.0));
        let frequency_param =
            RealtimeAudioParameter::new("frequency", frequency.clone(), frame_count);
        let gain = Arc::new(AtomicF64::new(1.0));
        let gain_param = RealtimeAudioParameter::new("gain", gain, frame_count);

        let mut parameters = DspParameters::new([frequency_param, gain_param]);

        let duration = Duration::from_secs(1);
        let actual = process(
            &mut processor,
            Duration::from_secs(1),
            sample_rate,
            frame_count,
            &mut parameters,
        );

        let expected = {
            let mut output = Vec::new();
            let sample_duration = (duration.as_secs_f64() * sample_rate as f64) as usize;
            let period = sample_rate as f64 / frequency.load(Ordering::Relaxed);
            let half_period = period / 2.0;
            for offset in 0..sample_duration {
                let phase = offset as f64 % period;
                let value = if phase < half_period {
                    phase / half_period
                } else {
                    1.0 - (phase - half_period) / half_period
                };

                output.push(value as f32);
            }

            output
        };

        assert_eq!(actual.len(), expected.len());

        for (actual, expected) in actual.iter().zip(expected.iter()) {
            assert_relative_eq!(actual, expected, epsilon = 1e-6);
        }
    }
}
