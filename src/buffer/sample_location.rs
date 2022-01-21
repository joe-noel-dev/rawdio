#[derive(Copy, Clone, Debug)]
pub struct SampleLocation {
    pub channel: usize,
    pub frame: usize,
}

impl SampleLocation {
    pub fn new(channel: usize, frame: usize) -> Self {
        Self { channel, frame }
    }
}
