use std::time::Duration;

use itertools::izip;

use super::compressor_parameters::get_range;
use crate::{
    dsp::mix_into_with_gains, effects::utility::EnvelopeFollower, graph::*, parameter::ParameterId,
    prelude::*, ProcessContext,
};

pub struct CompressorProcessor {
    envelopes: Vec<EnvelopeFollower>,
    gain_reduction_buffer: OwnedAudioBuffer,
}

impl CompressorProcessor {
    pub fn new(channel_count: usize, sample_rate: usize, maximum_frame_count: usize) -> Self {
        Self {
            envelopes: (0..channel_count)
                .map(|_| {
                    EnvelopeFollower::new(
                        sample_rate as f64,
                        Duration::from_secs_f64(get_range("attack").default() / 1_000.0),
                        Duration::from_secs_f64(get_range("release").default() / 1_000.0),
                    )
                })
                .collect(),
            gain_reduction_buffer: OwnedAudioBuffer::new(
                maximum_frame_count,
                channel_count,
                sample_rate,
            ),
        }
    }

    fn get_parameter_values<'a>(
        &self,
        parameter: ParameterId,
        frame_count: usize,
        parameters: &'a DspParameters,
    ) -> &'a [f32] {
        parameters.get_parameter_values(parameter, frame_count)
    }

    fn process_envelope(&mut self, context: &mut ProcessContext) {
        let attack = self.get_parameter_values(
            "attack",
            context.output_buffer.frame_count(),
            context.parameters,
        );

        let release = self.get_parameter_values(
            "release",
            context.output_buffer.frame_count(),
            context.parameters,
        );

        for channel in 0..context.input_buffer.channel_count() {
            let location = SampleLocation::channel(channel);
            let input_samples = context.input_buffer.get_channel_data(location);
            let gr_samples = self.gain_reduction_buffer.get_channel_data_mut(location);
            let envelope = &mut self.envelopes[channel];

            for (input_sample, gr_sample, attack, release) in izip!(
                input_samples.iter(),
                gr_samples.iter_mut(),
                attack.iter(),
                release.iter(),
            ) {
                envelope.set_attack_time(Duration::from_secs_f32(attack / 1_000.0));
                envelope.set_release_time(Duration::from_secs_f32(release / 1_000.0));
                *gr_sample = envelope.process(*input_sample);
            }
        }
    }

    fn calculate_gain_reduction(envelope: Level, knee: f32, threshold: f32, ratio: f32) -> f32 {
        debug_assert_ne!(ratio, 0.0);

        let envelope_db = envelope.as_db() as f32;
        let half_knee = 0.5 * knee;

        let output = if envelope_db < threshold + half_knee {
            envelope_db
        } else if envelope_db > threshold + half_knee {
            threshold + (envelope_db - threshold) / ratio
        } else {
            let excess = envelope_db - threshold + half_knee;
            envelope_db + ((1.0 / ratio - 1.0) * excess * excess) / (2.0 * knee)
        };

        Level::from_db_f32(output - envelope_db).as_linear_f32()
    }

    fn process_gain_reduction(&mut self, context: &mut ProcessContext) {
        let knee = self.get_parameter_values(
            "knee",
            context.output_buffer.frame_count(),
            context.parameters,
        );

        let threshold = self.get_parameter_values(
            "threshold",
            context.output_buffer.frame_count(),
            context.parameters,
        );

        let ratio = self.get_parameter_values(
            "ratio",
            context.output_buffer.frame_count(),
            context.parameters,
        );

        let wet = self.get_parameter_values(
            "wet",
            context.output_buffer.frame_count(),
            context.parameters,
        );

        let channel_count = context.output_buffer.channel_count();
        let frame_count = context.output_buffer.frame_count();

        for frame in 0..frame_count {
            let mut envelope = 0.0;

            for channel in 0..channel_count {
                let location = SampleLocation::new(channel, frame);
                envelope +=
                    self.gain_reduction_buffer.get_sample(location) as f64 / channel_count as f64;
            }

            let gain = Self::calculate_gain_reduction(
                Level::from_linear(envelope),
                knee[frame],
                threshold[frame],
                ratio[frame],
            );

            for channel in 0..channel_count {
                let location = SampleLocation::new(channel, frame);
                self.gain_reduction_buffer.set_sample(location, gain);
            }
        }

        context.output_buffer.copy_from(
            context.input_buffer,
            SampleLocation::origin(),
            SampleLocation::origin(),
            context.output_buffer.channel_count(),
            context.output_buffer.frame_count(),
        );

        let gain_reduction = self
            .gain_reduction_buffer
            .get_channel_data(SampleLocation::channel(0));
        let gain_reduction = &gain_reduction[..frame_count];
        context.output_buffer.apply_gain(gain_reduction);
        context.output_buffer.apply_gain(wet);
    }

    fn process_dry_signal(&mut self, context: &mut ProcessContext) {
        let dry = self.get_parameter_values(
            "dry",
            context.output_buffer.frame_count(),
            context.parameters,
        );

        let channel_count = context.output_buffer.channel_count();

        for channel in 0..channel_count {
            let location = SampleLocation::channel(channel);
            let input = context.input_buffer.get_channel_data(location);
            let output = context.output_buffer.get_channel_data_mut(location);
            mix_into_with_gains(input, output, dry);
        }
    }
}

impl DspProcessor for CompressorProcessor {
    fn process_audio(&mut self, context: &mut ProcessContext) {
        debug_assert_eq!(
            context.output_buffer.frame_count(),
            context.input_buffer.frame_count()
        );

        debug_assert_eq!(
            context.output_buffer.channel_count(),
            context.input_buffer.channel_count()
        );

        self.process_envelope(context);
        self.process_gain_reduction(context);
        self.process_dry_signal(context);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use approx::assert_relative_eq;
    use atomic_float::AtomicF64;

    use crate::{
        effects::compressor::compressor_parameters::get_range, parameter::RealtimeAudioParameter,
        BorrowedAudioBuffer, MutableBorrowedAudioBuffer, Timestamp,
    };

    use super::*;

    struct Fixture {
        compressor: CompressorProcessor,
        parameters: DspParameters,
        maximum_frame_count: usize,
    }

    impl Default for Fixture {
        fn default() -> Self {
            let channel_count = 1;
            let sample_rate = 48_000;
            let maximum_frame_count = 512;

            let params = [
                "attack",
                "release",
                "ratio",
                "threshold",
                "knee",
                "wet",
                "dry",
            ];

            let realtime_params = params.iter().map(|parameter| {
                let range = get_range(parameter);
                let value = Arc::new(AtomicF64::new(range.default()));
                RealtimeAudioParameter::new(parameter, value, maximum_frame_count)
            });

            let realtime_params = DspParameters::new(realtime_params);

            Self {
                compressor: CompressorProcessor::new(
                    channel_count,
                    sample_rate,
                    maximum_frame_count,
                ),
                parameters: realtime_params,
                maximum_frame_count,
            }
        }
    }

    impl Fixture {
        fn set_value(&mut self, parameter: ParameterId, value: f64) {
            self.parameters
                .get_parameter_mut(parameter)
                .set_value(value);
        }

        fn process(
            &mut self,
            input_signal: &dyn AudioBuffer,
            start_time: Timestamp,
        ) -> OwnedAudioBuffer {
            let frame_count = input_signal.frame_count();
            let channel_count = input_signal.channel_count();
            let sample_rate = input_signal.sample_rate();

            let mut output_buffer = OwnedAudioBuffer::new(frame_count, channel_count, sample_rate);

            for offset in (0..frame_count).step_by(self.maximum_frame_count) {
                let frames_this_time =
                    std::cmp::min(self.maximum_frame_count, frame_count - offset);

                let input_slice =
                    BorrowedAudioBuffer::slice_frames(input_signal, offset, frames_this_time);
                let mut output_slice = MutableBorrowedAudioBuffer::slice_frames(
                    &mut output_buffer,
                    offset,
                    frames_this_time,
                );

                let start_time = start_time.incremented_by_samples(offset, sample_rate);

                for parameter in self.parameters.iter_mut() {
                    parameter.1.process(&start_time, frame_count, sample_rate);
                }

                self.compressor.process_audio(&mut ProcessContext {
                    input_buffer: &input_slice,
                    output_buffer: &mut output_slice,
                    start_time: &start_time,
                    parameters: &self.parameters,
                });
            }

            output_buffer
        }
    }

    #[test]
    fn signal_unchanged_when_below_threshold() {
        let test_level = Level::from_db(-15.0);
        let frame_count = 48_000;
        let channel_count = 1;
        let sample_rate = 48_000;
        let frequency = 1_000.0;

        let test_signal = OwnedAudioBuffer::sine(
            frame_count,
            channel_count,
            sample_rate,
            frequency,
            test_level.as_linear(),
        );

        let mut fixture = Fixture::default();

        fixture.set_value("threshold", -12.0);

        let output = fixture.process(&test_signal, Timestamp::zero());

        let input_data = test_signal.get_channel_data(SampleLocation::origin());
        let output_data = output.get_channel_data(SampleLocation::origin());

        for (input_sample, output_sample) in izip!(input_data.iter(), output_data.iter()) {
            assert_relative_eq!(*input_sample, *output_sample, epsilon = 1e-6);
        }
    }

    #[test]
    fn signal_unchanged_when_fully_dry() {
        let test_level = Level::from_db(-15.0);
        let frame_count = 48_000;
        let channel_count = 1;
        let sample_rate = 48_000;
        let frequency = 1_000.0;

        let test_signal = OwnedAudioBuffer::sine(
            frame_count,
            channel_count,
            sample_rate,
            frequency,
            test_level.as_linear(),
        );

        let mut fixture = Fixture::default();

        fixture.set_value("threshold", -20.0);
        fixture.set_value("wet", 0.0);
        fixture.set_value("dry", 1.0);

        let output = fixture.process(&test_signal, Timestamp::zero());

        let input_data = test_signal.get_channel_data(SampleLocation::origin());
        let output_data = output.get_channel_data(SampleLocation::origin());

        for (input_sample, output_sample) in izip!(input_data.iter(), output_data.iter()) {
            assert_relative_eq!(*input_sample, *output_sample, epsilon = 1e-6);
        }
    }
}
