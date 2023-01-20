use crate::AudioBuffer;

pub trait AudioProcess {
    fn process(&mut self, output_buffer: &mut dyn AudioBuffer);
}
