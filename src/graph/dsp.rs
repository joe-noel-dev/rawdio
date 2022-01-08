use crate::{commands::id::Id, timestamp::Timestamp, utility::audio_buffer::AudioBuffer};

type ProcessFn = Box<dyn FnMut(&mut dyn AudioBuffer, &Timestamp) + Send + Sync>;

pub struct Dsp {
    id: Id,
    pub process: ProcessFn,
}

impl Dsp {
    pub fn new(id: Id, process: ProcessFn) -> Self {
        Self { id, process }
    }

    pub fn get_id(&self) -> Id {
        self.id
    }
}
