use super::sample_location::SampleLocation;

// FIXME: Replace implementations with SIMD instructions

pub trait AudioBuffer {
    fn num_channels(&self) -> usize;
    fn num_frames(&self) -> usize;
    fn sample_rate(&self) -> u32;
    fn clear(&mut self);

    fn set_sample(&mut self, sample_location: &SampleLocation, value: f32);
    fn add_sample(&mut self, sample_location: &SampleLocation, value: f32);
    fn get_sample(&self, sample_location: &SampleLocation) -> f32;

    fn add_from(
        &mut self,
        source_buffer: &dyn AudioBuffer,
        source_location: &SampleLocation,
        destination_location: &SampleLocation,
        num_channels: usize,
        num_frames: usize,
    ) {
        for frame in 0..num_frames {
            for channel in 0..num_channels {
                let source = SampleLocation::new(
                    channel + source_location.channel,
                    frame + source_location.frame,
                );

                let dest = SampleLocation::new(
                    channel + destination_location.channel,
                    frame + destination_location.frame,
                );

                self.set_sample(&dest, source_buffer.get_sample(&source));
            }
        }
    }
}
