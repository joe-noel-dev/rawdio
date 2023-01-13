use std::collections::HashMap;

use crate::OwnedAudioBuffer;

pub struct BufferPool<Identifier> {
    assigned_buffers: HashMap<Identifier, OwnedAudioBuffer>,
    free_buffers: Vec<OwnedAudioBuffer>,
    num_buffers: usize,
}

impl<Identifier> BufferPool<Identifier>
where
    Identifier: std::cmp::Eq + std::hash::Hash + Copy,
{
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

    pub fn get_buffer(&mut self, id: &Identifier) -> Option<OwnedAudioBuffer> {
        if let Some(assigned_buffer) = self.assigned_buffers.remove(id) {
            return Some(assigned_buffer);
        }

        self.free_buffers.pop()
    }

    pub fn clear_assignment(&mut self, id: &Identifier) {
        if let Some(buffer) = self.assigned_buffers.remove(id) {
            self.free_buffers.push(buffer);
        }
    }

    pub fn return_buffer(&mut self, buffer: OwnedAudioBuffer, id: &Identifier) {
        self.assigned_buffers.insert(*id, buffer);
    }

    pub fn clear_assignments(&mut self) {
        while !self.assigned_buffers.is_empty() {
            let id = *self.assigned_buffers.keys().next().unwrap();
            self.clear_assignment(&id);
        }
    }

    pub fn all_buffers_are_available(&self) -> bool {
        self.free_buffers.len() == self.num_buffers
    }
}
