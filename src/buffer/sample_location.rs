use std::ops::Range;

/// Specify the location in an audio buffer using the frame and channel index
#[derive(Copy, Clone, Debug)]
pub struct SampleLocation {
    /// Index of the channel
    pub channel: usize,

    /// Frame of the channel
    pub frame: usize,
}

impl SampleLocation {
    /// Create a new sample location with a channel and frame index
    pub fn new(channel: usize, frame: usize) -> Self {
        Self { channel, frame }
    }

    /// Create a new sample location at the first frame of a channel
    pub fn channel(channel: usize) -> Self {
        Self { channel, frame: 0 }
    }

    /// Create a new sample location at the first channel for a frame
    pub fn frame(frame: usize) -> Self {
        Self { channel: 0, frame }
    }

    /// Move this sample location by a number of frames
    pub fn offset_frames(self, frame_offset: usize) -> Self {
        self.with_frame(self.frame + frame_offset)
    }

    /// Move this sample location by a number of channels
    pub fn offset_channels(self, channel_offset: usize) -> Self {
        self.with_channel(self.channel + channel_offset)
    }

    /// Create a new sample location with the same frame at a different channel
    pub fn with_channel(&self, channel: usize) -> Self {
        Self {
            channel,
            frame: self.frame,
        }
    }

    /// Create a new sample location with the same channel at a different frame
    pub fn with_frame(&self, frame: usize) -> Self {
        Self {
            channel: self.channel,
            frame,
        }
    }

    /// Create a sample location at the origin (frame 0, sample 0)
    pub fn origin() -> Self {
        Self {
            channel: 0,
            frame: 0,
        }
    }
}

pub struct SampleRange {
    pub channel: usize,
    pub frame: usize,
    pub channel_count: usize,
    pub frame_count: usize,
}

impl SampleRange {
    pub fn new(channel: usize, frame: usize, channel_count: usize, frame_count: usize) -> Self {
        Self {
            channel,
            frame,
            channel_count,
            frame_count,
        }
    }

    pub fn channel_and_frame_count(channel_count: usize, frame_count: usize) -> Self {
        Self {
            channel: 0,
            frame: 0,
            channel_count,
            frame_count,
        }
    }

    pub fn channel_range(&self) -> Range<usize> {
        self.channel..self.channel + self.channel_count
    }

    pub fn frame_range(&self) -> Range<usize> {
        self.frame..self.frame + self.frame_count
    }
}
