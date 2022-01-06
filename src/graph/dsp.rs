use crate::{commands::id::Id, utility::audio_buffer::AudioBuffer};

pub trait Dsp {
    fn get_id(&self) -> Id;

    fn process(&mut self, output: &mut dyn AudioBuffer);
}
