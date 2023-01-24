use std::time::Duration;

use crate::SampleLocation;

pub trait AudioBuffer {
    fn fill_from_interleaved(
        &mut self,
        interleaved_data: &[f32],
        num_channels: usize,
        num_frames: usize,
    ) {
        let num_frames = num_frames.min(self.frame_count());
        let num_channels = num_channels.min(self.channel_count());

        for channel in 0..num_channels {
            let channel_data = self.get_channel_data_mut(SampleLocation::channel(channel));

            (0..num_frames).for_each(|frame| {
                let source_offset = frame * num_channels + channel;
                let sample_value = interleaved_data[source_offset];

                channel_data[frame] = sample_value;
            });
        }
    }

    fn copy_to_interleaved(
        &self,
        interleaved_data: &mut [f32],
        num_channels: usize,
        num_frames: usize,
    ) {
        let num_channels = num_channels.min(self.channel_count());
        let num_frames = num_frames.min(self.frame_count());

        for channel in 0..num_channels {
            let channel_data = self.get_channel_data(SampleLocation::channel(channel));

            (0..num_frames).for_each(|frame| {
                let sample_value = channel_data[frame];

                let destination_offset = frame * num_channels + channel;
                interleaved_data[destination_offset] = sample_value;
            });
        }
    }

    fn channel_count(&self) -> usize;

    fn frame_count(&self) -> usize;

    fn sample_rate(&self) -> usize;

    fn length_in_seconds(&self) -> f64 {
        self.frame_count() as f64 / self.sample_rate() as f64
    }

    fn duration(&self) -> Duration {
        Duration::from_secs_f64(self.length_in_seconds())
    }

    fn clear(&mut self) {
        self.fill_with_value(0.0_f32);
    }

    fn fill_channel_with_value(&mut self, channel: usize, value: f32) {
        let data = self.get_channel_data_mut(SampleLocation::new(channel, 0));
        data.fill(value);
    }

    fn fill_with_value(&mut self, value: f32) {
        for channel in 0..self.channel_count() {
            self.fill_channel_with_value(channel, value);
        }
    }

    fn channel_is_silent(&self, channel: usize) -> bool {
        let location = SampleLocation::new(channel, 0);
        let data = self.get_channel_data(location);
        data.iter().all(|sample| *sample == 0.0_f32)
    }

    fn get_channel_data(&self, sample_location: SampleLocation) -> &[f32];

    fn get_channel_data_mut(&mut self, sample_location: SampleLocation) -> &mut [f32];

    fn set_sample(&mut self, sample_location: SampleLocation, value: f32) {
        let data = self.get_channel_data_mut(sample_location);
        data[0] = value;
    }

    fn add_sample(&mut self, sample_location: SampleLocation, value: f32) {
        let value_before = self.get_sample(sample_location);
        self.set_sample(sample_location, value + value_before)
    }

    fn get_sample(&self, sample_location: SampleLocation) -> f32 {
        let data = self.get_channel_data(sample_location);
        data[0]
    }

    fn add_from(
        &mut self,
        source_buffer: &dyn AudioBuffer,
        source_location: SampleLocation,
        destination_location: SampleLocation,
        num_channels: usize,
        num_frames: usize,
    ) {
        for channel in 0..num_channels {
            let source = source_buffer.get_channel_data(source_location.offset_channels(channel));
            let source = &source[0..num_frames];

            let destination =
                self.get_channel_data_mut(destination_location.offset_channels(channel));
            let destination = &mut destination[0..num_frames];

            for (source_value, destination_value) in source.iter().zip(destination.iter_mut()) {
                *destination_value += *source_value;
            }
        }
    }

    fn copy_from(
        &mut self,
        source_buffer: &dyn AudioBuffer,
        source_location: SampleLocation,
        destination_location: SampleLocation,
        num_channels: usize,
        num_frames: usize,
    ) {
        for channel in 0..num_channels {
            let source = source_buffer.get_channel_data(source_location.offset_channels(channel));
            let source = &source[0..num_frames];

            let destination =
                self.get_channel_data_mut(destination_location.offset_channels(channel));
            let destination = &mut destination[0..num_frames];

            destination.copy_from_slice(source);
        }
    }

    fn frame_iter(&self) -> FrameIterator {
        FrameIterator {
            channel: 0,
            frame: 0,
            channel_count: self.channel_count(),
            frame_count: self.frame_count(),
        }
    }

    fn duplicate_channel(&mut self, source: SampleLocation, to_channel: usize, frame_count: usize);
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
