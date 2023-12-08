use std::{
    ops::{Add, Sub},
    time::Duration,
};

type FixedPoint = fixed::types::I32F32;

/// A fixed-point representation of a timestamp
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Timestamp {
    seconds: FixedPoint,
}

impl Default for Timestamp {
    fn default() -> Self {
        Self {
            seconds: FixedPoint::from_num(0.0),
        }
    }
}

impl Eq for Timestamp {}

impl PartialOrd for Timestamp {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Timestamp {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.seconds.cmp(&other.seconds)
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

impl Add for Timestamp {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            seconds: self.seconds.add(rhs.seconds),
        }
    }
}

impl Timestamp {
    /// Zero seconds
    pub fn zero() -> Self {
        Self {
            seconds: FixedPoint::ZERO,
        }
    }

    pub(crate) fn from_raw_i64(raw_value: i64) -> Self {
        Self {
            seconds: FixedPoint::from_bits(raw_value),
        }
    }

    pub(crate) fn as_raw_i64(&self) -> i64 {
        self.seconds.to_bits()
    }

    /// Create a timestamp from a number of seconds
    pub fn from_seconds(seconds: f64) -> Self {
        Self {
            seconds: FixedPoint::from_num(seconds),
        }
    }

    /// Create a timestamp from a number of samples at a sample rate
    pub fn from_samples(samples: f64, sample_rate: usize) -> Self {
        Self {
            seconds: FixedPoint::from_num(samples / sample_rate as f64),
        }
    }

    /// Create a timestamp from a duration
    pub fn from_duration(duration: Duration) -> Self {
        Self::from_seconds(duration.as_secs_f64())
    }

    /// Create a timestamp from a number of beats
    pub fn from_beats(beats: f64, tempo: f64) -> Self {
        debug_assert!(tempo > 0.0);

        let beat_frequency = tempo / 60.0;

        Self {
            seconds: FixedPoint::from_num(beats / beat_frequency),
        }
    }

    /// Get the number of seconds
    pub fn as_seconds(&self) -> f64 {
        self.seconds.to_num()
    }

    /// Get the number of samples at a sample rate
    pub fn as_samples(&self, sample_rate: usize) -> f64 {
        self.seconds.to_num::<f64>() * sample_rate as f64
    }

    /// Get the number of beats at a tempo
    pub fn as_beats(&self, tempo: f64) -> f64 {
        let beat_frequency = tempo / 60.0;
        self.seconds.to_num::<f64>() * beat_frequency
    }

    /// Increment by a number of samples
    pub fn incremented_by_samples(&self, sample_count: usize, sample_rate: usize) -> Self {
        *self + Self::from_samples(sample_count as f64, sample_rate)
    }

    /// Increment by a number of seconds
    pub fn incremented_by_seconds(&self, seconds: f64) -> Self {
        Self {
            seconds: self.seconds + FixedPoint::from_num(seconds),
        }
    }

    /// Increment by a number of beats
    pub fn incremented_by_beats(&self, beats: f64, tempo: f64) -> Self {
        *self + Self::from_beats(beats, tempo)
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;

    #[test]
    fn increment_by_samples() {
        let sample_rate = 44_100;
        let before = Timestamp::default();
        let after = before.incremented_by_samples(sample_rate, sample_rate);
        assert_relative_eq!(after.as_seconds() - before.as_seconds(), 1.0);
    }

    #[test]
    fn convert_from_beats() {
        let tempo = 120.0;
        let beats = 10.0;
        let timestamp = Timestamp::from_beats(beats, tempo);
        assert_relative_eq!(5.0, timestamp.as_seconds());
    }

    #[test]
    fn convert_to_beats() {
        let timestamp = Timestamp::from_seconds(5.0);
        let tempo = 120.0;
        let beats = timestamp.as_beats(tempo);
        assert_relative_eq!(beats, 10.0);
    }

    #[test]
    fn increment_by_beats() {
        let start = Timestamp::from_seconds(7.0);
        let tempo = 60.0;
        let beats = 9.0;
        let incremented = start.incremented_by_beats(beats, tempo);
        assert_eq!(incremented.as_seconds(), 16.0);
    }
}
