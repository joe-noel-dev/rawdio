use crate::buffer::audio_buffer::AudioBuffer;

pub trait AudioProcess {
    fn process(&mut self, output_buffer: &mut dyn AudioBuffer);
}
