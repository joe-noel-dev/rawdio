use crate::prelude::*;
use rand::Rng;
use std::ops::Range;

/// An audio buffer that owns its audio data
///
/// Create an `OwnedAudioBuffer` will allocate a vector with enough space to hold the audio data
#[repr(align(64))]
#[derive(Clone)]
pub struct OwnedAudioBuffer {
    data: Vec<f32>,
    channel_count: usize,
    frame_count: usize,
    sample_rate: usize,
}

impl OwnedAudioBuffer {
    /// Create an audio buffer with specified number of frames, channels, and at  the specified sample rate
    pub fn new(frame_count: usize, channel_count: usize, sample_rate: usize) -> Self {
        Self {
            data: vec![0.0; frame_count * channel_count],
            channel_count,
            frame_count,
            sample_rate,
        }
    }

    /// Create an audio buffer with audio from the specified slice
    ///
    /// The audio from `data` represents a single channel and will be copied to all channels
    pub fn from_slice(data: &[f32], channel_count: usize, sample_rate: usize) -> Self {
        let mut buffer = Self::new(data.len(), 1, sample_rate);

        for channel in 0..channel_count {
            buffer.fill_from_slice(data, SampleLocation::channel(channel));
        }

        buffer
    }

    /// Create a buffer by copying the data from another buffer
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

    /// Create a new buffer with the existing audio data, and copying in the data from `buffer`
    ///
    /// `buffer` should have the same number of channels and be at the same sample rate
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

    /// Create a buffer with random audio samples
    pub fn white_noise(frame_count: usize, channel_count: usize, sample_rate: usize) -> Self {
        let mut buffer = Self::new(frame_count, channel_count, sample_rate);

        let mut random_generator = rand::thread_rng();

        for frame in buffer.frame_iter() {
            let sample_value = random_generator.gen_range(-1.0..=1.0);
            buffer.set_sample(frame, sample_value);
        }

        buffer
    }

    /// Create a new buffer that has been padded to the specified length
    pub fn padded_to_length(&self, frame_count: usize) -> Self {
        let mut buffer = Self::new(frame_count, self.channel_count(), self.sample_rate());

        buffer.copy_from(
            self,
            SampleLocation::origin(),
            SampleLocation::origin(),
            self.channel_count(),
            self.frame_count(),
        );

        buffer
    }

    /// Create an audio buffer filled with a sine wave at the specified frequency
    pub fn sine(
        frame_count: usize,
        channel_count: usize,
        sample_rate: usize,
        frequency: f64,
        amplitude: f64,
    ) -> Self {
        debug_assert!(channel_count > 0);

        let mut buffer = Self::new(frame_count, channel_count, sample_rate);

        let channel = buffer.get_channel_data_mut(SampleLocation::origin());

        for (index, sample) in channel.iter_mut().enumerate() {
            let time = index as f64 / sample_rate as f64;
            *sample = (amplitude * (std::f64::consts::TAU * frequency * time).sin()) as f32;
        }

        for channel in 1..channel_count {
            let source_location = SampleLocation::channel(0);
            buffer.duplicate_channel(source_location, channel, frame_count);
        }

        buffer
    }

    /// Create an audio buffer that is value 0.0 until frame_count, then 1.0
    pub fn step(
        frame_count: usize,
        channel_count: usize,
        sample_rate: usize,
        step_range: Range<usize>,
    ) -> Self {
        let mut buffer = Self::new(frame_count, channel_count, sample_rate);

        let channel = buffer.get_channel_data_mut(SampleLocation::origin());

        for (index, sample) in channel.iter_mut().enumerate() {
            *sample = if step_range.contains(&index) {
                1.0
            } else {
                0.0
            };
        }

        for channel in 1..channel_count {
            let source_location = SampleLocation::channel(0);
            buffer.duplicate_channel(source_location, channel, frame_count);
        }

        buffer
    }

    fn get_sample_location_range(&self, sample_location: &SampleLocation) -> Range<usize> {
        let start = sample_location.channel * self.frame_count + sample_location.frame;
        let end = (sample_location.channel + 1) * self.frame_count;
        start..end
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
        let range = self.get_sample_location_range(&sample_location);
        &self.data[range]
    }

    fn get_channel_data_mut(&mut self, sample_location: SampleLocation) -> &mut [f32] {
        let range = self.get_sample_location_range(&sample_location);
        &mut self.data[range]
    }

    fn duplicate_channel(&mut self, source: SampleLocation, to_channel: usize, frame_count: usize) {
        let source_range = self.get_sample_location_range(&source);
        let destination_range = self.get_sample_location_range(&source.with_channel(to_channel));
        self.data.copy_within(
            source_range.start..source_range.start + frame_count,
            destination_range.start,
        );
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

    fn is_empty(buffer: &dyn AudioBuffer) -> bool {
        for channel in 0..buffer.channel_count() {
            let data = buffer.get_channel_data(SampleLocation::channel(channel));
            if !data.iter().all(|value| relative_eq!(*value, 0.0)) {
                return false;
            }
        }

        true
    }

    #[test]
    fn starts_empty() {
        let frame_count = 1000;
        let channel_count = 2;
        let sample_rate = 44100;
        let buffer = OwnedAudioBuffer::new(frame_count, channel_count, sample_rate);
        assert!(is_empty(&buffer));
    }

    #[test]
    fn clear_resets_all_samples() {
        let frame_count = 1000;
        let channel_count = 2;
        let sample_rate = 44100;
        let mut buffer = OwnedAudioBuffer::white_noise(frame_count, channel_count, sample_rate);

        assert!(!is_empty(&buffer));
        buffer.clear();
        assert!(is_empty(&buffer));
    }

    #[test]
    fn set_and_get_a_sample() {
        let frame_count = 1000;
        let channel_count = 2;
        let sample_rate = 44100;
        let mut buffer = OwnedAudioBuffer::new(frame_count, channel_count, sample_rate);

        let frame_index = 53;
        let channel_index = 1;
        let location = SampleLocation::new(channel_index, frame_index);

        let expected_sample = random_sample();
        buffer.set_sample(location, expected_sample);

        let actual_sample = buffer.get_sample(location);
        assert_eq!(expected_sample, actual_sample);
    }
}
