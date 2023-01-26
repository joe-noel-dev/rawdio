use std::time::Duration;

pub struct EnvelopeFollower {
    level: f32,
    attack_coefficient: f64,
    release_coefficient: f64,
}

fn calculate_coefficient(sample_rate: f64, time: Duration) -> f64 {
    (-1.0 / (sample_rate * time.as_secs_f64())).exp()
}

impl EnvelopeFollower {
    pub fn new(sample_rate: f64, attack_time: Duration, release_time: Duration) -> Self {
        Self {
            level: 0.0,
            attack_coefficient: calculate_coefficient(sample_rate, attack_time),
            release_coefficient: calculate_coefficient(sample_rate, release_time),
        }
    }

    pub fn process(&mut self, sample: f32) -> f32 {
        let input = sample.abs();

        let coefficient = if self.level < input {
            self.attack_coefficient
        } else {
            self.release_coefficient
        };

        self.level = (self.level * coefficient as f32) + input * (1.0_f32 - coefficient as f32);

        self.level
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;
    use itertools::Itertools;

    use super::*;

    #[test]
    fn test_0_attack_time() {
        let sample_rate = 48_000.0;
        let attack_time = Duration::from_millis(0);
        let release_time = Duration::from_millis(100);

        let mut envelope = EnvelopeFollower::new(sample_rate, attack_time, release_time);

        let frame_count = sample_rate as usize;

        let envelope: Vec<f32> = (0..frame_count)
            .map(|_| envelope.process(1.0_f32))
            .collect();

        for value in envelope {
            assert_relative_eq!(value, 1.0_f32);
        }
    }

    #[test]
    fn test_0_release_time() {
        let sample_rate = 48_000.0;
        let attack_time = Duration::from_millis(0);
        let release_time = Duration::from_millis(0);

        let mut envelope = EnvelopeFollower::new(sample_rate, attack_time, release_time);

        let frame_count = sample_rate as usize;

        envelope.process(1.0_f32);

        let envelope: Vec<f32> = (0..frame_count)
            .map(|_| envelope.process(0.0_f32))
            .collect();

        for value in envelope {
            assert_relative_eq!(value, 0.0_f32);
        }
    }

    #[test]
    fn test_attack_time() {
        let sample_rate = 48_000.0;
        let attack_time = Duration::from_millis(100);
        let release_time = Duration::from_millis(0);

        let mut envelope = EnvelopeFollower::new(sample_rate, attack_time, release_time);

        let frame_count = (sample_rate * attack_time.as_secs_f64()).ceil() as usize;

        let envelope: Vec<f32> = (0..frame_count - 1)
            .map(|_| envelope.process(1.0_f32))
            .collect();

        let max_value = 1.0_f32 - (-1.0_f32).exp();

        assert!(envelope.iter().tuple_windows().all(|(a, b)| a <= b));
        assert!(envelope
            .iter()
            .all(|value| 0.0 <= *value && *value <= max_value));
    }

    #[test]
    fn test_release_time() {
        let sample_rate = 48_000.0;
        let attack_time = Duration::from_millis(0);
        let release_time = Duration::from_millis(100);

        let mut envelope = EnvelopeFollower::new(sample_rate, attack_time, release_time);

        let frame_count = (sample_rate * release_time.as_secs_f64()).ceil() as usize;

        envelope.process(1.0_f32);

        let envelope: Vec<f32> = (0..frame_count - 1)
            .map(|_| envelope.process(0.0_f32))
            .collect();

        let min_value = (-1.0_f32).exp();

        assert!(envelope.iter().tuple_windows().all(|(a, b)| a >= b));
        assert!(envelope
            .iter()
            .all(|value| min_value <= *value && *value <= 1.0_f32));
    }
}
