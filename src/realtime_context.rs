use crate::utility::audio_buffer::AudioBuffer;

pub trait RealtimeContext {
    fn process(&mut self, data: &mut dyn AudioBuffer);
}
