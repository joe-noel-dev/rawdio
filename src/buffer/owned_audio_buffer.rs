use crate::{AudioBuffer, SampleLocation};

pub struct OwnedAudioBuffer {
    data: Vec<f32>,
    num_channels: usize,
    num_frames: usize,
    sample_rate: usize,
}

impl OwnedAudioBuffer {
    pub fn new(num_frames: usize, num_channels: usize, sample_rate: usize) -> Self {
        Self {
            data: vec![0.0; num_frames * num_channels],
            num_channels,
            num_frames,
            sample_rate,
        }
    }
}

impl AudioBuffer for OwnedAudioBuffer {
    fn num_channels(&self) -> usize {
        self.num_channels
    }

    fn num_frames(&self) -> usize {
        self.num_frames
    }

    fn sample_rate(&self) -> usize {
        self.sample_rate
    }

    fn get_data(&self, sample_location: SampleLocation) -> &[f32] {
        let start = sample_location.channel * self.num_frames + sample_location.frame;
        let end = (sample_location.channel + 1) * self.num_frames;
        &self.data[start..end]
    }

    fn get_data_mut(&mut self, sample_location: SampleLocation) -> &mut [f32] {
        let start = sample_location.channel * self.num_frames + sample_location.frame;
        let end = (sample_location.channel + 1) * self.num_frames;
        &mut self.data[start..end]
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use rand::Rng;

    fn random_sample() -> f32 {
        let mut generator = rand::thread_rng();
        generator.gen_range(-1.0_f32..=1.0_f32)
    }

    fn fill_with_noise(buffer: &mut dyn AudioBuffer) {
        for channel in 0..buffer.num_channels() {
            for frame in 0..buffer.num_frames() {
                buffer.set_sample(SampleLocation::new(channel, frame), random_sample());
            }
        }
    }

    fn is_empty(buffer: &dyn AudioBuffer) -> bool {
        for channel in 0..buffer.num_channels() {
            let data = buffer.get_data(SampleLocation::new(channel, 0));
            if !data.iter().all(|value| value.abs() < 1e-6) {
                return false;
            }
        }

        true
    }

    #[test]
    fn starts_empty() {
        let num_frames = 1000;
        let num_channels = 2;
        let sample_rate = 44100;
        let buffer = OwnedAudioBuffer::new(num_frames, num_channels, sample_rate);
        assert!(is_empty(&buffer));
    }

    #[test]
    fn clear_resets_all_samples() {
        let num_frames = 1000;
        let num_channels = 2;
        let sample_rate = 44100;
        let mut buffer = OwnedAudioBuffer::new(num_frames, num_channels, sample_rate);

        fill_with_noise(&mut buffer);
        assert!(!is_empty(&buffer));
        buffer.clear();
        assert!(is_empty(&buffer));
    }

    #[test]
    fn set_and_get_a_sample() {
        let num_frames = 1000;
        let num_channels = 2;
        let sample_rate = 44100;
        let mut buffer = OwnedAudioBuffer::new(num_frames, num_channels, sample_rate);

        let frame_index = 53;
        let channel_index = 1;
        let location = SampleLocation::new(channel_index, frame_index);

        let expected_sample = random_sample();
        buffer.set_sample(location, expected_sample);

        let actual_sample = buffer.get_sample(location);
        assert_eq!(expected_sample, actual_sample);
    }
}
