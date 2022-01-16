use std::collections::HashMap;

use crate::{
    buffer::{audio_buffer::AudioBuffer, owned_audio_buffer::OwnedAudioBuffer},
    graph::endpoint::Endpoint,
};

pub struct BufferPool {
    assigned_buffers: HashMap<Endpoint, OwnedAudioBuffer>,
    free_buffers: Vec<OwnedAudioBuffer>,
    num_buffers: usize,
}

impl BufferPool {
    pub fn with_capacity(
        num_buffers: usize,
        num_frames: usize,
        num_channels: usize,
        sample_rate: usize,
    ) -> Self {
        Self {
            assigned_buffers: HashMap::with_capacity(num_buffers),
            free_buffers: (0..num_buffers)
                .map(|_| OwnedAudioBuffer::new(num_frames, num_channels, sample_rate))
                .collect(),
            num_buffers,
        }
    }

    pub fn get_unassigned_buffer(&mut self) -> Option<OwnedAudioBuffer> {
        self.free_buffers.pop()
    }

    pub fn get_assigned_buffer(&mut self, for_endpoint: Endpoint) -> Option<OwnedAudioBuffer> {
        self.assigned_buffers.remove(&for_endpoint)
    }

    pub fn return_buffer(&mut self, mut buffer: OwnedAudioBuffer) {
        buffer.clear();
        self.free_buffers.push(buffer)
    }

    pub fn return_buffer_with_assignment(&mut self, buffer: OwnedAudioBuffer, endpoint: Endpoint) {
        self.assigned_buffers.insert(endpoint, buffer);
    }

    pub fn clear_assignments(&mut self) {
        while !self.assigned_buffers.is_empty() {
            let endpoint = *self.assigned_buffers.keys().next().unwrap();
            let buffer = self.assigned_buffers.remove(&endpoint).unwrap();
            self.free_buffers.push(buffer);
        }
    }

    pub fn all_buffers_are_available(&self) -> bool {
        self.free_buffers.len() == self.num_buffers
    }
}
