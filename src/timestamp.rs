use std::ops::Sub;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Timestamp {
    seconds: f64,
}

impl Default for Timestamp {
    fn default() -> Self {
        Self { seconds: 0.0 }
    }
}

impl Eq for Timestamp {}

impl PartialOrd for Timestamp {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.seconds.partial_cmp(&other.seconds)
    }
}

impl Ord for Timestamp {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap_or(std::cmp::Ordering::Equal)
    }
}

impl Sub for Timestamp {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            seconds: self.seconds.sub(rhs.seconds),
        }
    }
}

impl Timestamp {
    pub fn from_seconds(seconds: f64) -> Self {
        Self { seconds }
    }

    pub fn get_seconds(&self) -> f64 {
        self.seconds
    }

    pub fn get_samples(&self, sample_rate: usize) -> f64 {
        self.seconds * sample_rate as f64
    }

    pub fn incremented_by_samples(&self, num_samples: usize, sample_rate: usize) -> Self {
        Self {
            seconds: self.seconds + num_samples as f64 / sample_rate as f64,
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
        let sample_rate = 44_100;
        let before = Timestamp::default();
        let after = before.incremented_by_samples(sample_rate, sample_rate);
        assert_relative_eq!(after.get_seconds() - before.get_seconds(), 1.0);
    }
}
