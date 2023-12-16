use itertools::izip;

use crate::{graph::DspProcessor, SampleLocation};

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
        while self.phase > 1.0 {
            self.phase -= 1.0;
        }
    }

    fn get_value(&self) -> f64 {
        let offset = self.phase * self.wavetable.len() as f64;

        let offset_before = offset.floor() as usize;
        let offset_after = offset.ceil() as usize;

        let value_before = self.wavetable[offset_before];
        let value_after = if offset_after < self.wavetable.len() {
            self.wavetable[offset_after]
        } else {
            0.0
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
            self.increment_phase(*frequency, sample_rate);
            *sample = self.get_value() as f32;
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
