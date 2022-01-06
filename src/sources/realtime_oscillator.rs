use crate::{
    commands::id::Id,
    graph::dsp::Dsp,
    utility::audio_buffer::{AudioBuffer, SampleLocation},
};

pub struct RealtimeOscillator {
    id: Id,
    phase: f32,
    frequency: f32,
    sample_rate: f32,
}

impl Dsp for RealtimeOscillator {
    fn get_id(&self) -> Id {
        self.id
    }

    fn process(&mut self, output: &mut dyn AudioBuffer) {
        for frame in 0..output.num_frames() {
            self.increment_phase();
            let value = 0.5
                * (std::f32::consts::TAU * self.frequency * self.phase / self.sample_rate).sin();
            for channel in 0..output.num_channels() {
                let location = SampleLocation { channel, frame };
                output.set_sample(&location, value);
            }
        }
    }
}

impl RealtimeOscillator {
    pub fn new(id: Id, sample_rate: f32) -> Self {
        Self {
            id,
            phase: 0.0,
            frequency: 440.0,
            sample_rate,
        }
    }

    fn increment_phase(&mut self) {
        self.phase = (self.phase + 1.0) % self.sample_rate;
    }
}
