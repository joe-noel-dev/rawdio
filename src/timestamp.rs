#[derive(Clone, Copy)]
pub struct Timestamp {
    seconds: f64,
}

impl Default for Timestamp {
    fn default() -> Self {
        Self { seconds: 0.0 }
    }
}

impl Timestamp {
    fn with_seconds(seconds: f64) -> Self {
        Self { seconds }
    }

    pub fn incremented(&self, num_samples: usize, sample_rate: usize) -> Self {
        Self::with_seconds(self.seconds + num_samples as f64 / sample_rate as f64)
    }
}

impl Timestamp {
    pub fn get_seconds(&self) -> f64 {
        self.seconds
    }
}
