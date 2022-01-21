use super::{audio_buffer::AudioBuffer, sample_location::SampleLocation};

pub struct ImmutableAudioBufferSlice<'a> {
    buffer: &'a dyn AudioBuffer,
    offset: usize,
}

impl<'a> ImmutableAudioBufferSlice<'a> {
    pub fn _new(buffer: &'a dyn AudioBuffer, offset: usize) -> Self {
        if offset >= buffer.num_frames() {
            panic!("Invalid offset");
        }

        Self { buffer, offset }
    }

    fn translate_location(&self, sample_location: &SampleLocation) -> SampleLocation {
        SampleLocation::new(sample_location.channel, sample_location.frame + self.offset)
    }
}

impl<'a> AudioBuffer for ImmutableAudioBufferSlice<'a> {
    fn num_channels(&self) -> usize {
        self.buffer.num_channels()
    }

    fn num_frames(&self) -> usize {
        self.buffer.num_frames() - self.offset
    }

    fn sample_rate(&self) -> usize {
        self.buffer.sample_rate()
    }

    fn clear(&mut self) {
        debug_assert!(false)
    }

    fn set_sample(&mut self, _sample_location: &SampleLocation, _value: f32) {
        debug_assert!(false)
    }

    fn add_sample(&mut self, _sample_location: &SampleLocation, _value: f32) {
        debug_assert!(false)
    }

    fn get_sample(&self, sample_location: &SampleLocation) -> f32 {
        let new_location = self.translate_location(sample_location);
        self.buffer.get_sample(&new_location)
    }
}
