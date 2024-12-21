use crate::prelude::*;

use super::sample_location::SampleRange;

/// An AudioBuffer that borrows audio from a different audio buffer
///
/// This can be considered like a slice of another
pub struct BorrowedAudioBuffer<'a> {
    buffer: &'a dyn AudioBuffer,
    range: SampleRange,
}

impl<'a> BorrowedAudioBuffer<'a> {
    /// Create a slice of another audio buffer with a subset of the frames
    pub fn slice_frames(
        buffer: &'a dyn AudioBuffer,
        frame_offset: usize,
        frame_count: usize,
    ) -> Self {
        let channel_count = buffer.channel_count();
        Self::slice(
            buffer,
            SampleRange::new(0, frame_offset, channel_count, frame_count),
        )
    }

    /// Create a slice of another audio buffer with a subset of the channels
    pub fn slice_channels(
        buffer: &'a dyn AudioBuffer,
        channel_offset: usize,
        channel_count: usize,
    ) -> Self {
        let frame_count = buffer.frame_count();
        Self::slice(
            buffer,
            SampleRange::new(channel_offset, 0, frame_count, channel_count),
        )
    }

    /// Create a slice of another audio buffer with a subset of the channels and frames
    pub fn slice_channels_and_frames(
        buffer: &'a dyn AudioBuffer,
        frame_count: usize,
        channel_count: usize,
    ) -> Self {
        Self::slice(
            buffer,
            SampleRange::channel_and_frame_count(channel_count, frame_count),
        )
    }

    /// Create a slice of another audio buffer with a subset of the channels and frames
    pub fn slice(buffer: &'a dyn AudioBuffer, range: SampleRange) -> Self {
        assert!(buffer.range_is_valid(&range));
        Self { buffer, range }
    }
}

impl AudioBuffer for BorrowedAudioBuffer<'_> {
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

    fn get_channel_data_mut(&mut self, _sample_location: SampleLocation) -> &mut [f32] {
        panic!("Cannot get mutable channel data for an immutable audio buffer");
    }

    fn duplicate_channel(
        &mut self,
        _source: SampleLocation,
        _to_channel: usize,
        _frame_count: usize,
    ) {
        panic!("Cannot duplicate channel for an immutable audio buffer");
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

        let slice =
            BorrowedAudioBuffer::slice_frames(&original_buffer, slice_offset, slice_frame_count);
        let expected_location = SampleLocation::new(channel, frame - slice_offset);
        assert_relative_eq!(slice.get_sample(expected_location), value);
    }
}
