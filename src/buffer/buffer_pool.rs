use crate::{AudioBuffer, OwnedAudioBuffer};

pub struct BufferPool {
    free_buffers: Vec<OwnedAudioBuffer>,
}

impl BufferPool {
    pub fn new(
        buffer_count: usize,
        frame_count: usize,
        channel_count: usize,
        sample_rate: usize,
    ) -> Self {
        Self {
            free_buffers: (0..buffer_count)
                .map(|_| OwnedAudioBuffer::new(frame_count, channel_count, sample_rate))
                .collect(),
        }
    }

    pub fn remove(&mut self) -> Option<OwnedAudioBuffer> {
        self.free_buffers.pop()
    }

    pub fn add(&mut self, mut buffer: OwnedAudioBuffer) {
        buffer.clear();
        self.free_buffers.push(buffer);
    }
}
