pub struct SampleLocation {
    pub channel: usize,
    pub frame: usize,
}

impl SampleLocation {
    pub fn new(channel: usize, frame: usize) -> Self {
        Self { channel, frame }
    }

    pub fn get_channel(&self) -> usize {
        self.channel
    }

    pub fn get_frame(&self) -> usize {
        self.frame
    }
}
