use std::f64::consts::PI;

use crate::Level;

#[derive(Debug)]
pub struct BiquadCoefficients {
    a1: f64,
    a2: f64,
    b0: f64,
    b1: f64,
    b2: f64,
}

impl BiquadCoefficients {
    pub fn low_pass(center_frequency: f64, sample_rate: f64, q: f64) -> Self {
        let omega = omega(center_frequency, sample_rate);

        let b0 = (1.0 - omega.cos()) / 2.0;
        let b1 = 2.0 * b0;
        let b2 = b0;

        let alpha = omega.sin() / (2.0 * q);
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * omega.cos();
        let a2 = 1.0 - alpha;

        let scale = 1.0 / a0;

        Self {
            a1: a1 * scale,
            a2: a2 * scale,
            b0: b0 * scale,
            b1: b1 * scale,
            b2: b2 * scale,
        }
    }

    pub fn high_pass(center_frequency: f64, sample_rate: f64, q: f64) -> Self {
        let omega = omega(center_frequency, sample_rate);

        let b0 = (1.0 + omega.cos()) / 2.0;
        let b1 = -(1.0 + omega.cos());
        let b2 = b0;

        let alpha = omega.sin() / (2.0 * q);
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * omega.cos();
        let a2 = 1.0 - alpha;

        let scale = 1.0 / a0;

        Self {
            a1: a1 * scale,
            a2: a2 * scale,
            b0: b0 * scale,
            b1: b1 * scale,
            b2: b2 * scale,
        }
    }

    pub fn band_pass(center_frequency: f64, sample_rate: f64, q: f64) -> Self {
        let omega = omega(center_frequency, sample_rate);
        let alpha = omega.sin() / (2.0 * q);

        let b0 = alpha;
        let b1 = 0.0;
        let b2 = -alpha;

        let a0 = 1.0 + alpha;
        let a1 = -2.0 * omega.cos();
        let a2 = 1.0 - alpha;

        let scale = 1.0 / a0;

        Self {
            a1: a1 * scale,
            a2: a2 * scale,
            b0: b0 * scale,
            b1: b1 * scale,
            b2: b2 * scale,
        }
    }

    pub fn notch(center_frequency: f64, sample_rate: f64, q: f64) -> Self {
        let omega = omega(center_frequency, sample_rate);
        let alpha = omega.sin() / (2.0 * q);

        let b0 = 1.0;
        let b1 = -2.0 * omega.cos();
        let b2 = 1.0;

        let a0 = 1.0 + alpha;
        let a1 = -2.0 * omega.cos();
        let a2 = 1.0 - alpha;

        let scale = 1.0 / a0;

        Self {
            a1: a1 * scale,
            a2: a2 * scale,
            b0: b0 * scale,
            b1: b1 * scale,
            b2: b2 * scale,
        }
    }

    pub fn low_shelf(center_frequency: f64, sample_rate: f64, level: Level) -> Self {
        let a = level.as_gain().sqrt();

        let omega = omega(center_frequency, sample_rate);
        let beta = (2.0 * a).sqrt();

        let sin_omega = omega.sin();
        let cos_omega = omega.cos();

        let beta_sin_omega = beta * sin_omega;

        let b0 = a * ((a + 1.0) - (a - 1.0) * cos_omega + beta_sin_omega);
        let b1 = 2.0 * a * ((a - 1.0) - (a + 1.0) * cos_omega);
        let b2 = a * ((a + 1.0) - (a - 1.0) * cos_omega - beta_sin_omega);

        let a0 = (a + 1.0) + (a - 1.0) * cos_omega + beta_sin_omega;
        let a1 = -2.0 * ((a - 1.0) + (a + 1.0) * cos_omega);
        let a2 = (a + 1.0) + (a - 1.0) * cos_omega - beta_sin_omega;

        let scale = 1.0 / a0;

        Self {
            a1: a1 * scale,
            a2: a2 * scale,
            b0: b0 * scale,
            b1: b1 * scale,
            b2: b2 * scale,
        }
    }

    pub fn high_shelf(center_frequency: f64, sample_rate: f64, level: Level) -> Self {
        let a = level.as_gain().sqrt();

        let omega = omega(center_frequency, sample_rate);
        let beta = (a + a).sqrt();

        let sin_omega = omega.sin();
        let cos_omega = omega.cos();

        let beta_sin_omega = beta * sin_omega;

        let b0 = a * ((a + 1.0) + (a - 1.0) * cos_omega + beta_sin_omega);
        let b1 = -2.0 * a * ((a - 1.0) + (a + 1.0) * cos_omega);
        let b2 = a * ((a + 1.0) + (a - 1.0) * cos_omega - beta_sin_omega);

        let a0 = (a + 1.0) - (a - 1.0) * cos_omega + beta_sin_omega;
        let a1 = 2.0 * ((a - 1.0) - (a + 1.0) * cos_omega);
        let a2 = (a + 1.0) - (a - 1.0) * cos_omega - beta_sin_omega;

        let scale = 1.0 / a0;

        Self {
            a1: a1 * scale,
            a2: a2 * scale,
            b0: b0 * scale,
            b1: b1 * scale,
            b2: b2 * scale,
        }
    }

    pub fn a1(&self) -> f64 {
        self.a1
    }

    pub fn a2(&self) -> f64 {
        self.a2
    }

    pub fn b0(&self) -> f64 {
        self.b0
    }

    pub fn b1(&self) -> f64 {
        self.b1
    }

    pub fn b2(&self) -> f64 {
        self.b2
    }
}

fn omega(center_frequency: f64, sample_rate: f64) -> f64 {
    2.0 * PI * center_frequency / sample_rate
}

#[cfg(test)]
mod test {
    use approx::assert_relative_eq;

    use super::*;

    #[test]
    fn low_pass() {
        let coefficients = BiquadCoefficients::low_pass(5_000.0, 48_000.0, 1.414);
        assert_relative_eq!(coefficients.a1(), -1.30564969, epsilon = 1e-6);
        assert_relative_eq!(coefficients.a2(), 0.64573542, epsilon = 1e-6);
        assert_relative_eq!(coefficients.b0(), 0.08502143, epsilon = 1e-6);
        assert_relative_eq!(coefficients.b1(), 0.17004286, epsilon = 1e-6);
        assert_relative_eq!(coefficients.b2(), 0.08502143, epsilon = 1e-6);
    }

    #[test]
    fn high_pass() {
        let coefficients = BiquadCoefficients::high_pass(2_000.0, 96_000.0, 2.5);
        assert_relative_eq!(coefficients.a1(), -1.93244284, epsilon = 1e-6);
        assert_relative_eq!(coefficients.a2(), 0.94911781, epsilon = 1e-6);
        assert_relative_eq!(coefficients.b0(), 0.97039016, epsilon = 1e-6);
        assert_relative_eq!(coefficients.b1(), -1.94078033, epsilon = 1e-6);
        assert_relative_eq!(coefficients.b2(), 0.97039016, epsilon = 1e-6);
    }

    #[test]
    fn band_pass() {
        let coefficients = BiquadCoefficients::band_pass(3_000.0, 44_100.0, 6.0);
        assert_relative_eq!(coefficients.a1(), -1.75929661, epsilon = 1e-6);
        assert_relative_eq!(coefficients.a2(), 0.93321839, epsilon = 1e-6);
        assert_relative_eq!(coefficients.b0(), 0.03339080, epsilon = 1e-6);
        assert_relative_eq!(coefficients.b1(), 0.0, epsilon = 1e-6);
        assert_relative_eq!(coefficients.b2(), -0.03339080, epsilon = 1e-6);
    }

    #[test]
    fn notch() {
        let coefficients = BiquadCoefficients::notch(7_000.0, 88_200.0, 10.0);
        assert_relative_eq!(coefficients.a1(), -1.71542276, epsilon = 1e-6);
        assert_relative_eq!(coefficients.a2(), 0.95329153, epsilon = 1e-6);
        assert_relative_eq!(coefficients.b0(), 0.97664576, epsilon = 1e-6);
        assert_relative_eq!(coefficients.b1(), -1.71542276, epsilon = 1e-6);
        assert_relative_eq!(coefficients.b2(), 0.97664576, epsilon = 1e-6);
    }

    #[test]
    fn low_shelf() {
        let coefficients = BiquadCoefficients::low_shelf(80.0, 48_000.0, Level::from_db(6.0));
        println!("{coefficients:?}");
        assert_relative_eq!(coefficients.a1(), -1.98753939, epsilon = 1e-6);
        assert_relative_eq!(coefficients.a2(), 0.98761655, epsilon = 1e-6);
        assert_relative_eq!(coefficients.b0(), 1.00257352, epsilon = 1e-6);
        assert_relative_eq!(coefficients.b1(), -1.98750100, epsilon = 1e-6);
        assert_relative_eq!(coefficients.b2(), 0.98508142, epsilon = 1e-6);
    }

    #[test]
    fn high_shelf() {
        let coefficients = BiquadCoefficients::high_shelf(5_000.0, 96_000.0, Level::from_db(3.0));
        println!("{coefficients:?}");
        assert_relative_eq!(coefficients.a1(), -1.50372629, epsilon = 1e-6);
        assert_relative_eq!(coefficients.a2(), 0.60441923, epsilon = 1e-6);
        assert_relative_eq!(coefficients.b0(), 1.35784061, epsilon = 1e-6);
        assert_relative_eq!(coefficients.b1(), -2.14483965, epsilon = 1e-6);
        assert_relative_eq!(coefficients.b2(), 0.88769198, epsilon = 1e-6);
    }
}
