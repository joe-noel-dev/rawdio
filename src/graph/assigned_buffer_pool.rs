use std::collections::HashMap;

use crate::OwnedAudioBuffer;

pub struct AssignedBufferPool<Identifier> {
    assigned_buffers: HashMap<Identifier, OwnedAudioBuffer>,
}

impl<Identifier> AssignedBufferPool<Identifier>
where
    Identifier: std::cmp::Eq + std::hash::Hash + Copy,
{
    pub fn with_capacity(num_buffers: usize) -> Self {
        Self {
            assigned_buffers: HashMap::with_capacity(num_buffers),
        }
    }

    pub fn take(&mut self, id: &Identifier) -> Option<OwnedAudioBuffer> {
        self.assigned_buffers.remove(id)
    }

    pub fn add(&mut self, buffer: OwnedAudioBuffer, id: &Identifier) {
        self.assigned_buffers.insert(*id, buffer);
    }

    pub fn is_empty(&self) -> bool {
        self.assigned_buffers.is_empty()
    }

    pub fn get_next_id(&mut self) -> Option<Identifier> {
        self.assigned_buffers.keys().next().copied()
    }
}
