pub const MINUS_INFINITY_DECIBELS: f64 = -128.0;

/// A level that can be used to convert from linear to decibel representation
#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd)]
pub struct Level {
    linear: f64,
}

impl Level {
    /// Unity gain (1.0 / 0 dB)
    pub fn unity() -> Self {
        Level::from_linear(1.0)
    }

    /// Zero gain (0.0 / -inf dB)
    pub fn zero() -> Self {
        Level::from_linear(0.0)
    }

    /// Create a level from dB
    pub fn from_db(level_in_db: f64) -> Self {
        if level_in_db <= MINUS_INFINITY_DECIBELS {
            Self { linear: 0.0 }
        } else {
            Self {
                linear: 10.0_f64.powf(level_in_db / 20.0),
            }
        }
    }

    /// Create a level from dB
    pub fn from_db_f32(level_in_db: f32) -> Self {
        Self::from_db(level_in_db as f64)
    }

    /// Convert to dB
    pub fn as_db(&self) -> f64 {
        if self.linear <= 1e-9 {
            MINUS_INFINITY_DECIBELS
        } else {
            20.0 * self.linear.log10()
        }
    }

    /// Create a level from linear gain
    pub fn from_linear(linear_gain: f64) -> Self {
        Self {
            linear: linear_gain,
        }
    }

    /// Convert to linear gain
    pub fn as_linear(&self) -> f64 {
        self.linear
    }

    /// Convert to linear gain
    pub fn as_linear_f32(&self) -> f32 {
        self.linear as f32
    }

    /// Clamp the value between two other levels
    pub fn clamp(&self, min_value: &Self, max_value: &Self) -> Self {
        Self {
            linear: self.linear.clamp(min_value.linear, max_value.linear),
        }
    }

    /// Check if the value represents zero gain
    pub fn is_zero(&self) -> bool {
        self.linear.abs() < 1e-9
    }

    /// Check if the value represents unity gain
    pub fn is_unity(&self) -> bool {
        (self.linear - 1.0).abs() < 1e-9
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;

    #[test]
    fn db_to_linear() {
        let epsilon = 1e-2;
        assert_relative_eq!(Level::from_db(0.0).as_linear(), 1.0, epsilon = epsilon);
        assert_relative_eq!(Level::from_db(-6.0).as_linear(), 0.5, epsilon = epsilon);
        assert_relative_eq!(Level::from_db(-12.0).as_linear(), 0.25, epsilon = epsilon);
        assert_relative_eq!(Level::from_db(-200.0).as_linear(), 0.0, epsilon = epsilon);
    }

    #[test]
    fn linear_to_db() {
        let epsilon = 0.1;
        assert_relative_eq!(Level::from_linear(1.0).as_db(), 0.0, epsilon = epsilon);
        assert_relative_eq!(Level::from_linear(0.5).as_db(), -6.0, epsilon = epsilon);
        assert_relative_eq!(Level::from_linear(0.25).as_db(), -12.0, epsilon = epsilon);
        assert_relative_eq!(
            Level::from_linear(0.0).as_db(),
            MINUS_INFINITY_DECIBELS,
            epsilon = epsilon
        );
    }
}
