use crate::{
    commands::Id,
    graph::{DspParameters, DspProcessor},
    AudioBuffer, SampleLocation, Timestamp,
};

pub struct OscillatorProcessor {
    phase: f64,
    frequency_id: Id,
    gain_id: Id,
}

lazy_static! {
    static ref SINE_WAVE_TABLE: Vec<f64> = {
        let length = 8192;
        let mut values = Vec::with_capacity(length);

        for frame in 0..length {
            let time = frame as f64 / length as f64;
            let value = (std::f64::consts::TAU * time).sin();
            values.push(value);
        }

        values
    };
}

impl OscillatorProcessor {
    pub fn new(frequency_id: Id, gain_id: Id) -> Self {
        // ensure table is initialised off the realtime thread
        let _ = SINE_WAVE_TABLE[0];

        Self {
            phase: 0.0,
            frequency_id,
            gain_id,
        }
    }

    fn increment_phase(&mut self, frequency: f64, sample_rate: usize) {
        self.phase += frequency / (sample_rate as f64);
        while self.phase > 1.0 {
            self.phase -= 1.0;
        }
    }

    fn get_value(&self) -> f64 {
        let offset = self.phase * SINE_WAVE_TABLE.len() as f64;

        let offset_before = offset.floor() as usize;
        let offset_after = offset.ceil() as usize;

        let value_before = SINE_WAVE_TABLE[offset_before];
        let value_after = if offset_after < SINE_WAVE_TABLE.len() {
            SINE_WAVE_TABLE[offset_after]
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

        let frequency = match parameters.get(&self.frequency_id) {
            Some(param) => param,
            None => return,
        };

        let gain = match parameters.get(&self.gain_id) {
            Some(param) => param,
            None => return,
        };

        let num_frames = output_buffer.frame_count();
        let num_channels = output_buffer.channel_count();

        let frequency_values = frequency.get_values();
        let gain_values = gain.get_values();

        for frame in 0..num_frames {
            let frequency = frequency_values[frame];
            let gain = gain_values[frame];

            self.increment_phase(frequency, sample_rate);

            let value = gain * self.get_value();

            for channel in 0..num_channels {
                output_buffer.set_sample(SampleLocation::new(channel, frame), value as f32);
            }
        }
    }
}
