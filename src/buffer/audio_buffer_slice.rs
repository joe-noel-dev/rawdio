use super::{audio_buffer::AudioBuffer, sample_location::SampleLocation};

pub struct AudioBufferSlice<'a> {
    buffer: &'a mut dyn AudioBuffer,
    frame_offset: usize,
    num_frames: usize,
}

impl<'a> AudioBufferSlice<'a> {
    pub fn new(buffer: &'a mut dyn AudioBuffer, offset: usize, num_frames: usize) -> Self {
        assert!(offset + num_frames <= buffer.num_frames());

        Self {
            buffer,
            frame_offset: offset,
            num_frames,
        }
    }

    fn translate_location(&self, sample_location: SampleLocation) -> SampleLocation {
        debug_assert!(sample_location.frame < self.num_frames);

        SampleLocation::new(
            sample_location.channel,
            sample_location.frame + self.frame_offset,
        )
    }
}

impl<'a> AudioBuffer for AudioBufferSlice<'a> {
    fn num_channels(&self) -> usize {
        self.buffer.num_channels()
    }

    fn num_frames(&self) -> usize {
        self.num_frames
    }

    fn sample_rate(&self) -> usize {
        self.buffer.sample_rate()
    }

    fn clear(&mut self) {
        for frame in 0..self.num_frames {
            for channel in 0..self.num_channels() {
                self.set_sample(SampleLocation::new(channel, frame), 0.0);
            }
        }
    }

    fn set_sample(&mut self, sample_location: SampleLocation, value: f32) {
        let new_location = self.translate_location(sample_location);
        self.buffer.set_sample(new_location, value)
    }

    fn add_sample(&mut self, sample_location: SampleLocation, value: f32) {
        let new_location = self.translate_location(sample_location);
        self.buffer.add_sample(new_location, value)
    }

    fn get_sample(&self, sample_location: SampleLocation) -> f32 {
        let new_location = self.translate_location(sample_location);
        self.buffer.get_sample(new_location)
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use crate::OwnedAudioBuffer;

    use super::*;

    #[test]
    fn translates_location_when_getting_samples() {
        let mut original_buffer = OwnedAudioBuffer::new(1_000, 2, 44100);
        original_buffer.set_sample(SampleLocation::new(0, 54), 0.54);
        let slice = AudioBufferSlice::new(&mut original_buffer, 50, 50);
        assert_relative_eq!(slice.get_sample(SampleLocation::new(0, 4)), 0.54);
    }

    #[test]
    fn translates_location_when_setting_samples() {
        let mut original_buffer = OwnedAudioBuffer::new(1_000, 2, 44100);
        let mut slice = AudioBufferSlice::new(&mut original_buffer, 50, 50);
        slice.set_sample(SampleLocation::new(0, 12), 0.12);
        assert_relative_eq!(original_buffer.get_sample(SampleLocation::new(0, 62)), 0.12);
    }
}
