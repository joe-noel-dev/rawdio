use crate::AudioBuffer;

/// An audio process that can be processed with input and output audio samples
pub trait AudioProcess {
    /// Process audio
    fn process(&mut self, input_buffer: &dyn AudioBuffer, output_buffer: &mut dyn AudioBuffer);
}
