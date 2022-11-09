#[derive(Copy, Clone, Debug)]
pub struct SampleLocation {
    pub channel: usize,
    pub frame: usize,
}

impl SampleLocation {
    pub fn new(channel: usize, frame: usize) -> Self {
        Self { channel, frame }
    }

    pub fn offset_frames(&self, num_frames: usize) -> Self {
        Self {
            channel: self.channel,
            frame: self.frame + num_frames,
        }
    }

    pub fn with_channel(&self, channel: usize) -> Self {
        Self {
            channel,
            frame: self.frame,
        }
    }

    pub fn origin() -> Self {
        Self {
            channel: 0,
            frame: 0,
        }
    }
}
