use crate::SampleLocation;

pub trait AudioBuffer {
    fn fill_from_interleaved(
        &mut self,
        interleaved_data: &[f32],
        num_channels: usize,
        num_frames: usize,
    ) {
        for frame in 0..num_frames.min(self.num_frames()) {
            for channel in 0..num_channels.min(self.num_channels()) {
                let offset = frame * num_channels + channel;
                let sample_value = interleaved_data[offset];
                let location = SampleLocation::new(channel, frame);
                self.set_sample(location, sample_value);
            }
        }
    }

    fn copy_to_interleaved(
        &self,
        interleaved_data: &mut [f32],
        num_channels: usize,
        num_frames: usize,
    ) {
        assert!(num_channels * num_frames <= interleaved_data.len());

        for frame in 0..num_frames.min(self.num_frames()) {
            for channel in 0..num_channels.min(self.num_channels()) {
                let location = SampleLocation::new(channel, frame);
                let sample_value = self.get_sample(location);
                let offset = frame * num_channels + channel;
                interleaved_data[offset] = sample_value;
            }
        }
    }

    fn num_channels(&self) -> usize;

    fn num_frames(&self) -> usize;

    fn sample_rate(&self) -> usize;

    fn length_in_seconds(&self) -> f64 {
        self.num_frames() as f64 / self.sample_rate() as f64
    }

    fn clear(&mut self) {
        self.fill_with_value(0.0_f32);
    }

    fn fill_channel_with_value(&mut self, channel: usize, value: f32) {
        let data = self.get_data_mut(SampleLocation::new(channel, 0));
        data.fill(value);
    }

    fn fill_with_value(&mut self, value: f32) {
        for channel in 0..self.num_channels() {
            self.fill_channel_with_value(channel, value);
        }
    }

    fn channel_is_silent(&self, channel: usize) -> bool {
        let location = SampleLocation::new(channel, 0);
        let data = self.get_data(location);
        data.iter().all(|sample| *sample == 0.0_f32)
    }

    fn get_data(&self, sample_location: SampleLocation) -> &[f32];

    fn get_data_mut(&mut self, sample_location: SampleLocation) -> &mut [f32];

    fn set_sample(&mut self, sample_location: SampleLocation, value: f32) {
        let data = self.get_data_mut(sample_location);
        data[0] = value;
    }

    fn add_sample(&mut self, sample_location: SampleLocation, value: f32) {
        let value_before = self.get_sample(sample_location);
        self.set_sample(sample_location, value + value_before)
    }

    fn get_sample(&self, sample_location: SampleLocation) -> f32 {
        let data = self.get_data(sample_location);
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
            let source = source_buffer.get_data(source_location.offset_channels(channel));
            let source = &source[0..num_frames];

            let destination = self.get_data_mut(destination_location.offset_channels(channel));
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
            let source = source_buffer.get_data(source_location.offset_channels(channel));
            let source = &source[0..num_frames];

            let destination = self.get_data_mut(destination_location.offset_channels(channel));
            let destination = &mut destination[0..num_frames];

            destination.copy_from_slice(source);
        }
    }

    fn frame_iter(&self) -> FrameIterator {
        FrameIterator {
            channel: 0,
            frame: 0,
            num_channels: self.num_channels(),
            num_frames: self.num_frames(),
        }
    }
}

pub struct FrameIterator {
    channel: usize,
    frame: usize,
    num_channels: usize,
    num_frames: usize,
}

impl Iterator for FrameIterator {
    type Item = SampleLocation;

    fn next(&mut self) -> Option<Self::Item> {
        let location = if self.channel < self.num_channels && self.frame < self.num_frames {
            Some(SampleLocation::new(self.channel, self.frame))
        } else {
            None
        };

        self.frame += 1;

        if self.frame >= self.num_frames {
            self.channel += 1;
            self.frame = 0;
        }

        location
    }
}
