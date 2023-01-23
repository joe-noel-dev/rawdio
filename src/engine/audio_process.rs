use crate::AudioBuffer;

pub trait AudioProcess {
    fn process(&mut self, input_buffer: &dyn AudioBuffer, output_buffer: &mut dyn AudioBuffer);
}
