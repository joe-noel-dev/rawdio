use super::{audio_buffer::AudioBuffer, sample_location::SampleLocation};

pub struct AudioBufferSlice<'a> {
    buffer: &'a mut dyn AudioBuffer,
    offset: usize,
    num_frames: usize,
}

impl<'a> AudioBufferSlice<'a> {
    pub fn new(buffer: &'a mut dyn AudioBuffer, offset: usize, num_frames: usize) -> Self {
        if offset >= buffer.num_frames() {
            panic!("Invalid offset");
        }

        Self {
            buffer,
            offset,
            num_frames,
        }
    }

    fn translate_location(&self, sample_location: &SampleLocation) -> SampleLocation {
        SampleLocation::new(sample_location.channel, sample_location.frame + self.offset)
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
                self.set_sample(&SampleLocation::new(channel, frame), 0.0);
            }
        }
    }

    fn set_sample(&mut self, sample_location: &SampleLocation, value: f32) {
        let new_location = self.translate_location(sample_location);
        self.buffer.set_sample(&new_location, value)
    }

    fn add_sample(&mut self, sample_location: &SampleLocation, value: f32) {
        let new_location = self.translate_location(sample_location);
        self.buffer.add_sample(&new_location, value)
    }

    fn get_sample(&self, sample_location: &SampleLocation) -> f32 {
        let new_location = self.translate_location(sample_location);
        self.buffer.get_sample(&new_location)
    }
}
