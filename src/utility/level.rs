pub const MINUS_INFINITY_DECIBELS: f64 = -128.0;

/// A level that can be used to convert from linear to decibel representation
#[derive(Debug, Clone, Copy, Default)]
pub struct Level {
    gain: f64,
}

impl Level {
    /// Unity gain (1.0 / 0 dB)
    pub fn unity() -> Self {
        Level::from_gain(1.0)
    }

    /// Zero gain (0.0 / -inf dB)
    pub fn zero() -> Self {
        Level::from_gain(0.0)
    }

    /// Create a level from dB
    pub fn from_db(level_in_db: f64) -> Self {
        if level_in_db <= MINUS_INFINITY_DECIBELS {
            Self { gain: 0.0 }
        } else {
            Self {
                gain: 10.0_f64.powf(level_in_db / 20.0),
            }
        }
    }

    /// Convert to dB
    pub fn as_db(&self) -> f64 {
        if self.gain <= 1e-9 {
            MINUS_INFINITY_DECIBELS
        } else {
            20.0 * self.gain.log10()
        }
    }

    /// Create a level from linear gain
    pub fn from_gain(linear_gain: f64) -> Self {
        Self { gain: linear_gain }
    }

    /// Convert to linear gain
    pub fn as_gain(&self) -> f64 {
        self.gain
    }

    /// Convert to linear gain
    pub fn as_gain_f32(&self) -> f32 {
        self.gain as f32
    }

    /// Clamp the value between two other levels
    pub fn clamp(&self, min_value: &Self, max_value: &Self) -> Self {
        Self {
            gain: self.gain.clamp(min_value.gain, max_value.gain),
        }
    }

    /// Check if the value represents zero gain
    pub fn is_zero(&self) -> bool {
        self.gain.abs() < 1e-9
    }

    /// Check if the value represents unity gain
    pub fn is_unity(&self) -> bool {
        (self.gain - 1.0).abs() < 1e-9
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;

    #[test]
    fn db_to_gain() {
        let epsilon = 1e-2;
        assert_relative_eq!(Level::from_db(0.0).as_gain(), 1.0, epsilon = epsilon);
        assert_relative_eq!(Level::from_db(-6.0).as_gain(), 0.5, epsilon = epsilon);
        assert_relative_eq!(Level::from_db(-12.0).as_gain(), 0.25, epsilon = epsilon);
        assert_relative_eq!(Level::from_db(-200.0).as_gain(), 0.0, epsilon = epsilon);
    }

    #[test]
    fn gain_to_db() {
        let epsilon = 0.1;
        assert_relative_eq!(Level::from_gain(1.0).as_db(), 0.0, epsilon = epsilon);
        assert_relative_eq!(Level::from_gain(0.5).as_db(), -6.0, epsilon = epsilon);
        assert_relative_eq!(Level::from_gain(0.25).as_db(), -12.0, epsilon = epsilon);
        assert_relative_eq!(
            Level::from_gain(0.0).as_db(),
            MINUS_INFINITY_DECIBELS,
            epsilon = epsilon
        );
    }
}
