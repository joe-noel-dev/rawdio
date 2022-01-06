pub struct SampleLocation {
    pub channel: usize,
    pub frame: usize,
}

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
                let source = SampleLocation {
                    channel: channel + source_location.channel,
                    frame: frame + source_location.frame,
                };

                let dest = SampleLocation {
                    channel: channel + destination_location.channel,
                    frame: frame + destination_location.frame,
                };

                self.set_sample(&dest, source_buffer.get_sample(&source));
            }
        }
    }
}

pub struct BorrowedAudioBuffer<'a> {
    data: &'a mut [f32],
    num_channels: usize,
    sample_rate: u32,
}

impl<'a> BorrowedAudioBuffer<'a> {
    pub fn new(data: &'a mut [f32], num_channels: usize, sample_rate: u32) -> Self {
        Self {
            data,
            num_channels,
            sample_rate,
        }
    }
}

impl<'a> AudioBuffer for BorrowedAudioBuffer<'a> {
    fn num_channels(&self) -> usize {
        self.num_channels
    }

    fn num_frames(&self) -> usize {
        self.data.len() / self.num_channels
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn clear(&mut self) {
        for value in self.data.iter_mut() {
            *value = 0.0;
        }
    }

    fn set_sample(&mut self, sample_location: &SampleLocation, value: f32) {
        debug_assert!(sample_location.channel < self.num_channels);
        debug_assert!(sample_location.frame < self.num_frames());
        self.data[sample_location.frame * self.num_channels + sample_location.channel] = value;
    }

    fn add_sample(&mut self, sample_location: &SampleLocation, value: f32) {
        let value_before = self.get_sample(sample_location);
        self.set_sample(sample_location, value + value_before)
    }

    fn get_sample(&self, sample_location: &SampleLocation) -> f32 {
        debug_assert!(sample_location.channel < self.num_channels);
        debug_assert!(sample_location.frame < self.num_frames());
        self.data[sample_location.frame * self.num_channels + sample_location.channel]
    }
}

pub struct OwnedAudioBuffer {
    data: Vec<f32>,
    num_channels: usize,
    sample_rate: u32,
}

impl<'a> OwnedAudioBuffer {
    pub fn new(data: Vec<f32>, num_channels: usize, sample_rate: u32) -> Self {
        Self {
            data,
            num_channels,
            sample_rate,
        }
    }
}

impl AudioBuffer for OwnedAudioBuffer {
    fn num_channels(&self) -> usize {
        self.num_channels
    }

    fn num_frames(&self) -> usize {
        self.data.len() / self.num_channels
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn clear(&mut self) {
        for value in self.data.iter_mut() {
            *value = 0.0;
        }
    }

    fn set_sample(&mut self, sample_location: &SampleLocation, value: f32) {
        debug_assert!(sample_location.channel < self.num_channels);
        debug_assert!(sample_location.frame < self.num_frames());
        self.data[sample_location.frame * self.num_channels + sample_location.channel] = value;
    }

    fn add_sample(&mut self, sample_location: &SampleLocation, value: f32) {
        let value_before = self.get_sample(sample_location);
        self.set_sample(sample_location, value + value_before)
    }

    fn get_sample(&self, sample_location: &SampleLocation) -> f32 {
        debug_assert!(sample_location.channel < self.num_channels);
        debug_assert!(sample_location.frame < self.num_frames());
        self.data[sample_location.frame * self.num_channels + sample_location.channel]
    }
}

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
        SampleLocation {
            channel: sample_location.channel,
            frame: sample_location.frame + self.offset,
        }
    }
}

impl<'a> AudioBuffer for AudioBufferSlice<'a> {
    fn num_channels(&self) -> usize {
        self.buffer.num_channels()
    }

    fn num_frames(&self) -> usize {
        self.buffer.num_frames() - self.offset
    }

    fn sample_rate(&self) -> u32 {
        self.buffer.sample_rate()
    }

    fn clear(&mut self) {
        for frame in 0..self.num_frames {
            for channel in 0..self.num_channels() {
                self.set_sample(&SampleLocation { channel, frame }, 0.0);
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

pub struct ImmutableAudioBufferSlice<'a> {
    buffer: &'a dyn AudioBuffer,
    offset: usize,
}

impl<'a> ImmutableAudioBufferSlice<'a> {
    pub fn new(buffer: &'a dyn AudioBuffer, offset: usize) -> Self {
        if offset >= buffer.num_frames() {
            panic!("Invalid offset");
        }

        Self { buffer, offset }
    }

    fn translate_location(&self, sample_location: &SampleLocation) -> SampleLocation {
        SampleLocation {
            channel: sample_location.channel,
            frame: sample_location.frame + self.offset,
        }
    }
}

impl<'a> AudioBuffer for ImmutableAudioBufferSlice<'a> {
    fn num_channels(&self) -> usize {
        self.buffer.num_channels()
    }

    fn num_frames(&self) -> usize {
        self.buffer.num_frames() - self.offset
    }

    fn sample_rate(&self) -> u32 {
        self.buffer.sample_rate()
    }

    fn clear(&mut self) {}

    fn set_sample(&mut self, _sample_location: &SampleLocation, _value: f32) {}

    fn add_sample(&mut self, _sample_location: &SampleLocation, _value: f32) {}

    fn get_sample(&self, sample_location: &SampleLocation) -> f32 {
        let new_location = self.translate_location(sample_location);
        self.buffer.get_sample(&new_location)
    }
}
