use super::{audio_buffer::AudioBuffer, sample_location::SampleLocation};

pub struct OwnedAudioBuffer {
    data: Vec<f32>,
    num_channels: usize,
    sample_rate: usize,
}

impl OwnedAudioBuffer {
    pub fn new(num_frames: usize, num_channels: usize, sample_rate: usize) -> Self {
        Self {
            data: vec![0.0; num_frames * num_channels],
            num_channels,
            sample_rate,
        }
    }

    fn get_offset(&self, sample_location: &SampleLocation) -> usize {
        debug_assert!(sample_location.channel < self.num_channels);
        debug_assert!(sample_location.frame < self.num_frames());
        sample_location.frame * self.num_channels + sample_location.channel
    }
}

impl AudioBuffer for OwnedAudioBuffer {
    fn num_channels(&self) -> usize {
        self.num_channels
    }

    fn num_frames(&self) -> usize {
        self.data.len() / self.num_channels
    }

    fn sample_rate(&self) -> usize {
        self.sample_rate
    }

    fn clear(&mut self) {
        self.data.fill(0.0);
    }

    fn set_sample(&mut self, sample_location: &SampleLocation, value: f32) {
        let offset = self.get_offset(sample_location);
        self.data[offset] = value;
    }

    fn add_sample(&mut self, sample_location: &SampleLocation, value: f32) {
        let value_before = self.get_sample(sample_location);
        self.set_sample(sample_location, value + value_before)
    }

    fn get_sample(&self, sample_location: &SampleLocation) -> f32 {
        let offset = self.get_offset(sample_location);
        self.data[offset]
    }
}
