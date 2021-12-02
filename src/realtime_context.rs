pub trait RealtimeContext {
    fn process(&mut self, data: &mut [f32], num_channels: usize);
}
