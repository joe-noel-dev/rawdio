use crate::{AudioBuffer, SampleLocation};

#[derive(Clone)]
pub struct OwnedAudioBuffer {
    data: Vec<f32>,
    channel_count: usize,
    frame_count: usize,
    sample_rate: usize,
}

impl OwnedAudioBuffer {
    pub fn new(frame_count: usize, channel_count: usize, sample_rate: usize) -> Self {
        Self {
            data: vec![0.0; frame_count * channel_count],
            channel_count,
            frame_count,
            sample_rate,
        }
    }

    pub fn from_buffer(buffer: &dyn AudioBuffer) -> Self {
        let mut new_buffer = Self::new(
            buffer.frame_count(),
            buffer.channel_count(),
            buffer.sample_rate(),
        );

        new_buffer.copy_from(
            buffer,
            SampleLocation::origin(),
            SampleLocation::origin(),
            buffer.channel_count(),
            buffer.frame_count(),
        );

        new_buffer
    }

    pub fn extended_with_buffer(&self, buffer: &dyn AudioBuffer) -> Self {
        assert_eq!(buffer.channel_count(), self.channel_count());
        assert_eq!(buffer.sample_rate(), self.sample_rate());

        let mut new_buffer = Self::new(
            self.frame_count() + buffer.frame_count(),
            self.channel_count(),
            self.sample_rate(),
        );

        new_buffer.copy_from(
            self,
            SampleLocation::origin(),
            SampleLocation::origin(),
            self.channel_count(),
            self.frame_count(),
        );

        new_buffer.copy_from(
            buffer,
            SampleLocation::origin(),
            SampleLocation::frame(self.frame_count()),
            buffer.channel_count(),
            buffer.frame_count(),
        );

        new_buffer
    }

    fn get_sample_location_bounds(&self, sample_location: &SampleLocation) -> (usize, usize) {
        let start = sample_location.channel * self.frame_count + sample_location.frame;
        let end = (sample_location.channel + 1) * self.frame_count;
        (start, end)
    }
}

impl AudioBuffer for OwnedAudioBuffer {
    fn channel_count(&self) -> usize {
        self.channel_count
    }

    fn frame_count(&self) -> usize {
        self.frame_count
    }

    fn sample_rate(&self) -> usize {
        self.sample_rate
    }

    fn get_channel_data(&self, sample_location: SampleLocation) -> &[f32] {
        let (start, end) = self.get_sample_location_bounds(&sample_location);
        &self.data[start..end]
    }

    fn get_channel_data_mut(&mut self, sample_location: SampleLocation) -> &mut [f32] {
        let (start, end) = self.get_sample_location_bounds(&sample_location);
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
        for channel in 0..buffer.channel_count() {
            for frame in 0..buffer.frame_count() {
                buffer.set_sample(SampleLocation::new(channel, frame), random_sample());
            }
        }
    }

    fn is_empty(buffer: &dyn AudioBuffer) -> bool {
        for channel in 0..buffer.channel_count() {
            let data = buffer.get_channel_data(SampleLocation::new(channel, 0));
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
