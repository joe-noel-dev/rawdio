use itertools::izip;

use crate::{
    commands::Id,
    graph::{DspParameters, DspProcessor},
    utility::macros::unwrap_or_return,
    AudioBuffer, SampleLocation, Timestamp,
};

pub struct OscillatorProcessor {
    phase: f64,
    frequency_id: Id,
    gain_id: Id,
    wavetable: Vec<f64>,
}

impl OscillatorProcessor {
    pub fn new(frequency_id: Id, gain_id: Id, wavetable: Vec<f64>) -> Self {
        Self {
            phase: 0.0,
            frequency_id,
            gain_id,
            wavetable,
        }
    }

    fn increment_phase(&mut self, frequency: f64, sample_rate: usize) {
        self.phase += frequency / (sample_rate as f64);
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
    fn process_audio(
        &mut self,
        _input_buffer: &dyn AudioBuffer,
        output_buffer: &mut dyn AudioBuffer,
        _start_time: &Timestamp,
        parameters: &DspParameters,
    ) {
        let sample_rate = output_buffer.sample_rate();

        let frequency_values = unwrap_or_return!(parameters.get(&self.frequency_id)).get_values();
        let gain_values = unwrap_or_return!(parameters.get(&self.gain_id)).get_values();

        let channel_count = output_buffer.channel_count();

        let location = SampleLocation::channel(0);
        let channel_data = output_buffer.get_channel_data_mut(location);

        for (sample, frequency, gain) in izip!(
            channel_data.iter_mut(),
            frequency_values.iter(),
            gain_values.iter()
        ) {
            self.increment_phase(*frequency, sample_rate);
            *sample = (*gain * self.get_value()) as f32;
        }

        (1..channel_count).for_each(|channel| {
            let location = SampleLocation::channel(0);
            output_buffer.duplicate_channel(location, channel, output_buffer.frame_count());
        });
    }
}
