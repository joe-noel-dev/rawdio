use super::{audio_buffer::AudioBuffer, sample_location::SampleLocation};

pub struct OwnedAudioBuffer {
    data: Vec<f32>,
    num_channels: usize,
    sample_rate: u32,
}

impl OwnedAudioBuffer {
    pub fn new(num_frames: usize, num_channels: usize, sample_rate: u32) -> Self {
        Self {
            data: vec![0.0; num_frames * num_channels],
            num_channels,
            sample_rate,
        }
    }
}

impl AudioBuffer for OwnedAudioBuffer {
    fn num_channels(&self) -> usize {
        self.num_channels
    }

    fn num_frames(&self) -> usize {
        self.data.len() / self.num_channels
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn clear(&mut self) {
        for value in self.data.iter_mut() {
            *value = 0.0;
        }
    }

    fn set_sample(&mut self, sample_location: &SampleLocation, value: f32) {
        debug_assert!(sample_location.channel < self.num_channels);
        debug_assert!(sample_location.frame < self.num_frames());
        self.data[sample_location.frame * self.num_channels + sample_location.channel] = value;
    }

    fn add_sample(&mut self, sample_location: &SampleLocation, value: f32) {
        let value_before = self.get_sample(sample_location);
        self.set_sample(sample_location, value + value_before)
    }

    fn get_sample(&self, sample_location: &SampleLocation) -> f32 {
        debug_assert!(sample_location.channel < self.num_channels);
        debug_assert!(sample_location.frame < self.num_frames());
        self.data[sample_location.frame * self.num_channels + sample_location.channel]
    }
}
