use crate::{AudioBuffer, SampleLocation};

pub struct BorrowedAudioBuffer<'a> {
    buffer: &'a mut dyn AudioBuffer,
    frame_offset: usize,
    frame_count: usize,
    channel_offset: usize,
    channel_count: usize,
}

impl<'a> BorrowedAudioBuffer<'a> {
    pub fn slice_frames(
        buffer: &'a mut dyn AudioBuffer,
        frame_offset: usize,
        frame_count: usize,
    ) -> Self {
        assert!(frame_offset + frame_count <= buffer.num_frames());
        let num_channels = buffer.num_channels();
        Self::slice(buffer, frame_offset, frame_count, 0, num_channels)
    }

    pub fn slice_channels(
        buffer: &'a mut dyn AudioBuffer,
        channel_offset: usize,
        channel_count: usize,
    ) -> Self {
        assert!(channel_offset + channel_count <= buffer.num_channels());
        let num_frames = buffer.num_frames();
        Self::slice(buffer, 0, num_frames, channel_offset, channel_count)
    }

    pub fn slice(
        buffer: &'a mut dyn AudioBuffer,
        frame_offset: usize,
        frame_count: usize,
        channel_offset: usize,
        channel_count: usize,
    ) -> Self {
        assert!(frame_offset + frame_count <= buffer.num_frames());
        assert!(channel_offset + channel_count <= buffer.num_channels());

        Self {
            buffer,
            frame_offset,
            frame_count,
            channel_offset,
            channel_count,
        }
    }
}

impl<'a> AudioBuffer for BorrowedAudioBuffer<'a> {
    fn num_channels(&self) -> usize {
        self.channel_count
    }

    fn num_frames(&self) -> usize {
        self.frame_count
    }

    fn sample_rate(&self) -> usize {
        self.buffer.sample_rate()
    }

    fn get_data(&self, sample_location: SampleLocation) -> &[f32] {
        let data = self.buffer.get_data(
            sample_location
                .offset_frames(self.frame_offset)
                .offset_channels(self.channel_offset),
        );
        let end = self.frame_count - sample_location.frame;
        &data[0..end]
    }

    fn get_data_mut(&mut self, sample_location: SampleLocation) -> &mut [f32] {
        let data = self.buffer.get_data_mut(
            sample_location
                .offset_frames(self.frame_offset)
                .offset_channels(self.channel_offset),
        );
        let end = self.frame_count - sample_location.frame;
        &mut data[0..end]
    }
}

#[cfg(test)]
mod tests {

    use crate::OwnedAudioBuffer;

    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn translates_location_when_getting_samples() {
        let num_frames = 1_000;
        let num_channels = 2;
        let sample_rate = 44_100;
        let mut original_buffer = OwnedAudioBuffer::new(num_frames, num_channels, sample_rate);

        let channel = 0;
        let frame = 54;
        let location = SampleLocation::new(channel, frame);

        let value = 0.54;
        original_buffer.set_sample(location, value);

        let slice_offset = 50;
        let slice_num_frames = 100;
        let slice =
            BorrowedAudioBuffer::slice_frames(&mut original_buffer, slice_offset, slice_num_frames);
        let expected_location = SampleLocation::new(channel, frame - slice_offset);
        assert_relative_eq!(slice.get_sample(expected_location), value);
    }

    #[test]
    fn translates_location_when_setting_samples() {
        let num_frames = 1_000;
        let num_channels = 2;
        let sample_rate = 44_100;
        let mut original_buffer = OwnedAudioBuffer::new(num_frames, num_channels, sample_rate);

        let slice_offset = 50;
        let slice_num_frames = 100;
        let mut slice =
            BorrowedAudioBuffer::slice_frames(&mut original_buffer, slice_offset, slice_num_frames);

        let channel = 0;
        let offset_frame = 12;
        let offset_location = SampleLocation::new(channel, offset_frame);
        let sample_value = 0.12;
        slice.set_sample(offset_location, sample_value);

        let original_location = SampleLocation::new(channel, offset_frame + slice_offset);
        assert_relative_eq!(original_buffer.get_sample(original_location), sample_value);
    }
}
