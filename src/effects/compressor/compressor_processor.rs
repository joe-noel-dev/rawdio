use std::{collections::HashMap, time::Duration};

use itertools::izip;

use crate::{
    commands::Id,
    dsp::mix_into_with_gains,
    effects::utility::EnvelopeFollower,
    graph::{DspParameters, DspProcessor},
    AudioBuffer, Level, OwnedAudioBuffer, ProcessContext, SampleLocation,
};

use super::compressor_parameters::{attack_range, release_range, CompressorParameter};

pub struct CompressorProcessor {
    ids: HashMap<CompressorParameter, Id>,
    envelopes: Vec<EnvelopeFollower>,
    gain_reduction_buffer: OwnedAudioBuffer,
}

impl CompressorProcessor {
    pub fn new(
        channel_count: usize,
        sample_rate: usize,
        maximum_frame_count: usize,
        ids: HashMap<CompressorParameter, Id>,
    ) -> Self {
        Self {
            ids,
            envelopes: (0..channel_count)
                .map(|_| {
                    EnvelopeFollower::new(
                        sample_rate as f64,
                        Duration::from_secs_f64(attack_range().default() / 1_000.0),
                        Duration::from_secs_f64(release_range().default() / 1_000.0),
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
        parameter: CompressorParameter,
        frame_count: usize,
        parameters: &'a DspParameters,
    ) -> &'a [f32] {
        let id = self
            .ids
            .get(&parameter)
            .unwrap_or_else(|| panic!("Parameter ID not found: {parameter:#?}"));

        parameters.get_parameter_values(*id, frame_count)
    }

    fn process_envelope(&mut self, context: &mut ProcessContext) {
        let attack = self.get_parameter_values(
            CompressorParameter::Attack,
            context.output_buffer.frame_count(),
            context.parameters,
        );

        let release = self.get_parameter_values(
            CompressorParameter::Release,
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

        Level::from_db_f32(output - envelope_db).as_gain_f32()
    }

    fn process_gain_reduction(&mut self, context: &mut ProcessContext) {
        let knee = self.get_parameter_values(
            CompressorParameter::Knee,
            context.output_buffer.frame_count(),
            context.parameters,
        );

        let threshold = self.get_parameter_values(
            CompressorParameter::Threshold,
            context.output_buffer.frame_count(),
            context.parameters,
        );

        let ratio = self.get_parameter_values(
            CompressorParameter::Ratio,
            context.output_buffer.frame_count(),
            context.parameters,
        );

        let makeup = self.get_parameter_values(
            CompressorParameter::Makeup,
            context.output_buffer.frame_count(),
            context.parameters,
        );

        let wet = self.get_parameter_values(
            CompressorParameter::WetLevel,
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
                Level::from_gain(envelope),
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
        context.output_buffer.apply_gain(makeup);
        context.output_buffer.apply_gain(wet);
    }

    fn process_dry_signal(&mut self, context: &mut ProcessContext) {
        let dry = self.get_parameter_values(
            CompressorParameter::DryLevel,
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
        ids: HashMap<CompressorParameter, Id>,
        parameters: DspParameters,
        maximum_frame_count: usize,
    }

    impl Default for Fixture {
        fn default() -> Self {
            let channel_count = 1;
            let sample_rate = 48_000;
            let maximum_frame_count = 512;

            let params = [
                CompressorParameter::Attack,
                CompressorParameter::Release,
                CompressorParameter::Ratio,
                CompressorParameter::Threshold,
                CompressorParameter::Knee,
                CompressorParameter::Makeup,
                CompressorParameter::WetLevel,
                CompressorParameter::DryLevel,
            ];

            let mut ids = HashMap::new();
            params.iter().for_each(|param| {
                ids.insert(*param, Id::generate());
            });

            let realtime_parameters: Vec<RealtimeAudioParameter> = params
                .iter()
                .map(|parameter| {
                    let id = ids[parameter];
                    let value = get_range(*parameter);
                    let value = AtomicF64::new(value.default());
                    let value = Arc::new(value);
                    RealtimeAudioParameter::new(id, value, maximum_frame_count)
                })
                .collect();

            Self {
                compressor: CompressorProcessor::new(
                    channel_count,
                    sample_rate,
                    maximum_frame_count,
                    ids.clone(),
                ),
                ids,
                parameters: DspParameters::from(realtime_parameters),
                maximum_frame_count,
            }
        }
    }

    impl Fixture {
        fn set_value(&mut self, parameter: CompressorParameter, value: f64) {
            let id = self.ids.get(&parameter).expect("Parameter not found");
            let realtime_parameter = self.parameters.get_parameter_mut(*id);
            realtime_parameter.set_value(value);
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

            let mut remaining = frame_count;
            let mut offset = 0;

            while remaining > 0 {
                let frames_this_time = self.maximum_frame_count.min(remaining);

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

                offset += frames_this_time;
                remaining -= frames_this_time;
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
            test_level.as_gain(),
        );

        let mut fixture = Fixture::default();

        fixture.set_value(CompressorParameter::Threshold, -12.0);

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
            test_level.as_gain(),
        );

        let mut fixture = Fixture::default();

        fixture.set_value(CompressorParameter::Threshold, -20.0);
        fixture.set_value(CompressorParameter::WetLevel, 0.0);
        fixture.set_value(CompressorParameter::DryLevel, 1.0);

        let output = fixture.process(&test_signal, Timestamp::zero());

        let input_data = test_signal.get_channel_data(SampleLocation::origin());
        let output_data = output.get_channel_data(SampleLocation::origin());

        for (input_sample, output_sample) in izip!(input_data.iter(), output_data.iter()) {
            assert_relative_eq!(*input_sample, *output_sample, epsilon = 1e-6);
        }
    }
}
