use super::sample_location::SampleRange;
use crate::{dsp::*, prelude::*};
use std::time::Duration;

/// An `AudioBuffer` represents floating point audio data for a number of channels
///
/// Audio is always de-interleaved
pub trait AudioBuffer {
    /// Fill the buffer with interleaved audio
    ///
    /// The length of `interleaved_data` should be at least `channel_count * frame_count`
    fn fill_from_interleaved(
        &mut self,
        interleaved_data: &[f32],
        channel_count: usize,
        frame_count: usize,
    ) {
        debug_assert!(interleaved_data.len() >= channel_count * frame_count);

        let frame_count = frame_count.min(self.frame_count());
        let channel_count = channel_count.min(self.channel_count());

        for channel in 0..channel_count {
            let channel_data = self.get_channel_data_mut(SampleLocation::channel(channel));

            (0..frame_count).for_each(|frame| {
                let source_offset = frame * channel_count + channel;
                let sample_value = interleaved_data[source_offset];

                channel_data[frame] = sample_value;
            });
        }
    }

    /// Copy the data to an interleaved buffer
    ///
    /// The length of `interleaved_data` should be at least `channel_count * frame_count`
    fn copy_to_interleaved(
        &self,
        interleaved_data: &mut [f32],
        channel_count: usize,
        frame_count: usize,
    ) {
        debug_assert!(interleaved_data.len() >= channel_count * frame_count);

        let channel_count = channel_count.min(self.channel_count());
        let frame_count = frame_count.min(self.frame_count());

        for channel in 0..channel_count {
            let channel_data = self.get_channel_data(SampleLocation::channel(channel));

            (0..frame_count).for_each(|frame| {
                let sample_value = channel_data[frame];

                let destination_offset = frame * channel_count + channel;
                interleaved_data[destination_offset] = sample_value;
            });
        }
    }

    /// Get the number of channels in this buffer
    fn channel_count(&self) -> usize;

    /// Get the number of frames in this buffer
    fn frame_count(&self) -> usize;

    /// Get the sample rate of this buffer
    fn sample_rate(&self) -> usize;

    /// Get the length represented by this buffer in seconds
    fn length_in_seconds(&self) -> f64 {
        self.frame_count() as f64 / self.sample_rate() as f64
    }

    /// Get the duration of this buffer in seconds
    fn duration(&self) -> Duration {
        Duration::from_secs_f64(self.length_in_seconds())
    }

    /// Clear the audio in this buffer
    fn clear(&mut self) {
        self.fill_with_value(0.0_f32);
    }

    /// Verify that a sample range is valid
    fn range_is_valid(&self, range: &SampleRange) -> bool {
        if range.channel + range.channel_count > self.channel_count() {
            return false;
        }

        if range.frame + range.frame_count > self.frame_count() {
            return false;
        }

        true
    }

    /// Clear the audio in a range of samples
    fn clear_range(&mut self, range: &SampleRange) {
        debug_assert!(self.range_is_valid(range));

        for channel in range.channel..range.channel + range.channel_count {
            let data = self.get_channel_data_mut(SampleLocation::new(channel, range.frame));
            let data = &mut data[..range.frame_count];
            data.fill(0.0_f32);
        }
    }

    /// Fill a channel with a value
    fn fill_channel_with_value(&mut self, channel: usize, value: f32) {
        let data = self.get_channel_data_mut(SampleLocation::channel(channel));
        data.fill(value);
    }

    /// Fill the entire buffer with a value
    fn fill_with_value(&mut self, value: f32) {
        for channel in 0..self.channel_count() {
            self.fill_channel_with_value(channel, value);
        }
    }

    /// Check if a channel is silent
    ///
    /// This can be used to perform optimisations
    fn channel_is_silent(&self, channel: usize) -> bool {
        let location = SampleLocation::channel(channel);
        let data = self.get_channel_data(location);
        data.iter().all(|sample| *sample == 0.0_f32)
    }

    /// Get a slice representing the audio data of a particular channel
    fn get_channel_data(&self, sample_location: SampleLocation) -> &[f32];

    /// Get a mutable slice representing the audio data of a particular channel
    fn get_channel_data_mut(&mut self, sample_location: SampleLocation) -> &mut [f32];

    /// Set a sample in the buffer
    fn set_sample(&mut self, sample_location: SampleLocation, value: f32) {
        let data = self.get_channel_data_mut(sample_location);
        data[0] = value;
    }

    /// Add a sample into the buffer
    fn add_sample(&mut self, sample_location: SampleLocation, value: f32) {
        let value_before = self.get_sample(sample_location);
        self.set_sample(sample_location, value + value_before)
    }

    /// Get a sample from the buffer
    fn get_sample(&self, sample_location: SampleLocation) -> f32 {
        let data = self.get_channel_data(sample_location);
        data[0]
    }

    /// Mix audio from one buffer into another buffer
    fn add_from(
        &mut self,
        source_buffer: &dyn AudioBuffer,
        source_location: SampleLocation,
        destination_location: SampleLocation,
        channel_count: usize,
        frame_count: usize,
    ) {
        if frame_count == 0 {
            return;
        }

        if channel_count == 0 {
            return;
        }

        for channel in 0..channel_count {
            let source = source_buffer.get_channel_data(source_location.offset_channels(channel));
            let source = &source[..frame_count];

            let destination =
                self.get_channel_data_mut(destination_location.offset_channels(channel));
            let destination = &mut destination[..frame_count];

            mix_into(source, destination);
        }
    }

    /// Mix audio from one buffer into another buffer, apply a constant gain to each sample
    fn add_from_with_gain(
        &mut self,
        source_buffer: &dyn AudioBuffer,
        source_location: SampleLocation,
        destination_location: SampleLocation,
        channel_count: usize,
        frame_count: usize,
        gain: f32,
    ) {
        if frame_count == 0 {
            return;
        }

        if channel_count == 0 {
            return;
        }

        for channel in 0..channel_count {
            let source = source_buffer.get_channel_data(source_location.offset_channels(channel));
            let source = &source[..frame_count];

            let destination =
                self.get_channel_data_mut(destination_location.offset_channels(channel));
            let destination = &mut destination[..frame_count];

            mix_into_with_gain(source, destination, gain);
        }
    }

    /// Copy audio from one buffer into another
    ///
    /// This will replace the audio in the destination buffer
    fn copy_from(
        &mut self,
        source_buffer: &dyn AudioBuffer,
        source_location: SampleLocation,
        destination_location: SampleLocation,
        channel_count: usize,
        frame_count: usize,
    ) {
        for channel in 0..channel_count {
            let source = source_buffer.get_channel_data(source_location.offset_channels(channel));
            let source = &source[..frame_count];

            let destination =
                self.get_channel_data_mut(destination_location.offset_channels(channel));
            let destination = &mut destination[..frame_count];

            destination.copy_from_slice(source);
        }
    }

    /// Copy audio data within the buffer
    fn copy_within(&mut self, source_range: &SampleRange, destination_frame: usize) {
        for channel in source_range.channel_range() {
            let data = self.get_channel_data_mut(SampleLocation::channel(channel));
            data.copy_within(source_range.frame_range(), destination_frame);
        }
    }

    /// Apply gain to all channels the buffer
    ///
    /// Rather than a fixed gain, this uses a 'table' to represent the gain values
    ///
    /// The length of `gain` should be the same as the frame count of the buffer
    fn apply_gain(&mut self, gain: &[f32]) {
        debug_assert_eq!(gain.len(), self.frame_count());

        if gain.iter().all(|gain| relative_eq!(*gain, 0.0)) {
            self.clear();
            return;
        }

        if gain.iter().all(|gain| relative_eq!(*gain, 1.0)) {
            return;
        }

        for channel in 0..self.channel_count() {
            let channel_data = self.get_channel_data_mut(SampleLocation::channel(channel));
            multiply(channel_data, gain);
        }
    }

    /// Apply a single gain value to a whole channel
    fn apply_gain_value(&mut self, range: &SampleRange, gain: f32) {
        for channel in range.channel_range() {
            let channel_data = self.get_channel_data_mut(SampleLocation::new(channel, range.frame));
            let channel_data = &mut channel_data[..range.frame_count];

            if relative_eq!(gain, 0.0) {
                channel_data.fill(0.0);
                return;
            }

            if relative_eq!(gain, 1.0) {
                return;
            }

            multiply_by_value(channel_data, gain);
        }
    }

    /// Get an iterator to iteraotr over every sample in the buffer
    ///
    /// Incrementing the iterator will go to the next frame in the same channel.
    /// When it reaches the end of the channel, it will go onto the next channel.
    fn frame_iter(&self) -> FrameIterator {
        FrameIterator {
            channel: 0,
            frame: 0,
            channel_count: self.channel_count(),
            frame_count: self.frame_count(),
        }
    }

    /// Duplicate the audio data from one channel to a different channel
    fn duplicate_channel(&mut self, source: SampleLocation, to_channel: usize, frame_count: usize);

    /// Copy audio from a different audio buffer at a different sample rate
    ///
    /// This will perform an interpolation-based sample rate conversion
    fn sample_rate_convert_from(
        &mut self,
        audio_buffer: &dyn AudioBuffer,
        source_location: SampleLocation,
        destination_location: SampleLocation,
        channel_count: usize,
    ) {
        let ratio = audio_buffer.sample_rate() as f32 / self.sample_rate() as f32;

        for channel_offset in 0..channel_count {
            let source_data =
                audio_buffer.get_channel_data(source_location.offset_channels(channel_offset));
            let destination_data =
                self.get_channel_data_mut(destination_location.offset_channels(channel_offset));

            destination_data
                .iter_mut()
                .enumerate()
                .for_each(|(index, sample)| {
                    let source_index = index as f32 * ratio;

                    let sample_before = (source_index.floor() as usize).min(source_data.len() - 1);
                    let sample_after = (source_index.ceil() as usize).min(source_data.len() - 1);

                    if sample_before == sample_after {
                        *sample = source_data[sample_before];
                    }

                    let sample_before = source_data[sample_before];
                    let sample_after = source_data[sample_after];

                    let sample_after_amount = source_index - source_index.floor();
                    *sample = (1.0_f32 - sample_after_amount) * sample_before
                        + sample_after_amount * sample_after;
                });
        }
    }

    /// Fill a channel from a slice
    fn fill_from_slice(&mut self, audio_data: &[f32], location: SampleLocation) {
        self.get_channel_data_mut(location)
            .copy_from_slice(audio_data);
    }
}

pub struct FrameIterator {
    channel: usize,
    frame: usize,
    channel_count: usize,
    frame_count: usize,
}

impl Iterator for FrameIterator {
    type Item = SampleLocation;

    fn next(&mut self) -> Option<Self::Item> {
        let location = if self.channel < self.channel_count && self.frame < self.frame_count {
            Some(SampleLocation::new(self.channel, self.frame))
        } else {
            None
        };

        self.frame += 1;

        if self.frame >= self.frame_count {
            self.channel += 1;
            self.frame = 0;
        }

        location
    }
}
