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

    pub fn remove(&mut self, id: &Identifier) -> Option<OwnedAudioBuffer> {
        self.assigned_buffers.remove(id)
    }

    pub fn add(&mut self, buffer: OwnedAudioBuffer, id: &Identifier) {
        self.assigned_buffers.insert(*id, buffer);
    }

    pub fn is_empty(&self) -> bool {
        self.assigned_buffers.is_empty()
    }

    pub fn has(&self, id: &Identifier) -> bool {
        self.assigned_buffers.contains_key(id)
    }

    pub fn remove_next(&mut self) -> Option<(Identifier, OwnedAudioBuffer)> {
        let id = match self.assigned_buffers.keys().next() {
            Some(id) => *id,
            None => return None,
        };

        let buffer = self.remove(&id).expect("Buffer not found");

        Some((id, buffer))
    }
}
