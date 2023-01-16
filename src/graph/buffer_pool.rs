use crate::OwnedAudioBuffer;

pub struct BufferPool {
    free_buffers: Vec<OwnedAudioBuffer>,
}

impl BufferPool {
    pub fn new(
        num_buffers: usize,
        num_frames: usize,
        num_channels: usize,
        sample_rate: usize,
    ) -> Self {
        Self {
            free_buffers: (0..num_buffers)
                .map(|_| OwnedAudioBuffer::new(num_frames, num_channels, sample_rate))
                .collect(),
        }
    }

    pub fn remove(&mut self) -> Option<OwnedAudioBuffer> {
        self.free_buffers.pop()
    }

    pub fn add(&mut self, buffer: OwnedAudioBuffer) {
        self.free_buffers.push(buffer);
    }
}
