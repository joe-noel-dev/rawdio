use crate::buffer::audio_buffer::AudioBuffer;

pub trait AudioProcess {
    fn process(&mut self, data: &mut dyn AudioBuffer);
}
