use super::sample_location::SampleRange;
use crate::prelude::*;

/// A mutable buffer that refers to a portion of another buffer
pub struct MutableBorrowedAudioBuffer<'a> {
    buffer: &'a mut dyn AudioBuffer,
    range: SampleRange,
}

impl<'a> MutableBorrowedAudioBuffer<'a> {
    /// Create a slice of another audio buffer with a subset of frames
    pub fn slice_frames(
        buffer: &'a mut dyn AudioBuffer,
        frame_offset: usize,
        frame_count: usize,
    ) -> Self {
        let channel_count = buffer.channel_count();
        Self::slice(
            buffer,
            SampleRange::new(0, frame_offset, channel_count, frame_count),
        )
    }

    /// Create a slice of another audio buffer with a subset of channels
    pub fn slice_channels(
        buffer: &'a mut dyn AudioBuffer,
        channel_offset: usize,
        channel_count: usize,
    ) -> Self {
        let frame_count = buffer.frame_count();
        Self::slice(
            buffer,
            SampleRange::new(channel_offset, 0, channel_count, frame_count),
        )
    }

    /// Create a slice of another audio buffer with a subset of channels and frames
    pub fn slice_channels_and_frames(
        buffer: &'a mut dyn AudioBuffer,
        frame_count: usize,
        channel_count: usize,
    ) -> Self {
        Self::slice(
            buffer,
            SampleRange::channel_and_frame_count(channel_count, frame_count),
        )
    }

    /// Create a slice of another audio buffer with a subset of channels and frames
    pub fn slice(buffer: &'a mut dyn AudioBuffer, range: SampleRange) -> Self {
        assert!(buffer.range_is_valid(&range));
        Self { buffer, range }
    }
}

impl<'a> AudioBuffer for MutableBorrowedAudioBuffer<'a> {
    fn channel_count(&self) -> usize {
        self.range.channel_count
    }

    fn frame_count(&self) -> usize {
        self.range.frame_count
    }

    fn sample_rate(&self) -> usize {
        self.buffer.sample_rate()
    }

    fn get_channel_data(&self, sample_location: SampleLocation) -> &[f32] {
        let data = self.buffer.get_channel_data(
            sample_location
                .offset_frames(self.range.frame)
                .offset_channels(self.range.channel),
        );
        let end = self.range.frame_count - sample_location.frame;
        &data[0..end]
    }

    fn get_channel_data_mut(&mut self, sample_location: SampleLocation) -> &mut [f32] {
        let data = self.buffer.get_channel_data_mut(
            sample_location
                .offset_frames(self.range.frame)
                .offset_channels(self.range.channel),
        );
        let end = self.range.frame_count - sample_location.frame;
        &mut data[0..end]
    }

    fn duplicate_channel(&mut self, source: SampleLocation, to_channel: usize, frame_count: usize) {
        self.buffer.duplicate_channel(
            source
                .offset_frames(self.range.frame)
                .offset_channels(self.range.channel),
            to_channel + self.range.channel,
            frame_count,
        );
    }
}

#[cfg(test)]
mod tests {

    use crate::OwnedAudioBuffer;

    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn translates_location_when_getting_samples() {
        let frame_count = 1_000;
        let channel_count = 2;
        let sample_rate = 44_100;

        let mut original_buffer = OwnedAudioBuffer::new(frame_count, channel_count, sample_rate);

        let channel = 0;
        let frame = 54;
        let location = SampleLocation::new(channel, frame);

        let value = 0.54;
        original_buffer.set_sample(location, value);

        let slice_offset = 50;
        let slice_frame_count = 100;
        let slice = MutableBorrowedAudioBuffer::slice_frames(
            &mut original_buffer,
            slice_offset,
            slice_frame_count,
        );
        let expected_location = SampleLocation::new(channel, frame - slice_offset);
        assert_relative_eq!(slice.get_sample(expected_location), value);
    }

    #[test]
    fn translates_location_when_setting_samples() {
        let frame_count = 1_000;
        let channel_count = 2;
        let sample_rate = 44_100;

        let mut original_buffer = OwnedAudioBuffer::new(frame_count, channel_count, sample_rate);

        let slice_offset = 50;
        let slice_frame_count = 100;
        let mut slice = MutableBorrowedAudioBuffer::slice_frames(
            &mut original_buffer,
            slice_offset,
            slice_frame_count,
        );

        let channel = 0;
        let offset_frame = 12;
        let offset_location = SampleLocation::new(channel, offset_frame);
        let sample_value = 0.12;
        slice.set_sample(offset_location, sample_value);

        let original_location = SampleLocation::new(channel, offset_frame + slice_offset);
        assert_relative_eq!(original_buffer.get_sample(original_location), sample_value);
    }
}
