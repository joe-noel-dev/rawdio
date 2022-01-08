#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Timestamp {
    seconds: f64,
}

impl Default for Timestamp {
    fn default() -> Self {
        Self { seconds: 0.0 }
    }
}

impl Timestamp {
    pub fn from_seconds(seconds: f64) -> Self {
        Self { seconds }
    }

    pub fn get_seconds(&self) -> f64 {
        self.seconds
    }

    pub fn incremented_by_samples(&self, num_samples: usize, sample_rate: f64) -> Self {
        Self {
            seconds: self.seconds + num_samples as f64 / sample_rate,
        }
    }

    pub fn incremented_by_seconds(&self, num_seconds: f64) -> Self {
        Self {
            seconds: self.seconds + num_seconds,
        }
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;
    #[test]
    fn it_increments() {
        let sample_rate = 44100.0;
        let before = Timestamp::default();
        let after = before.incremented_by_samples(sample_rate as usize, sample_rate);
        assert_relative_eq!(after.get_seconds() - before.get_seconds(), 1.0);
    }
}
