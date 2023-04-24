use std::time::Duration;

pub struct Fade {
    values: Vec<f32>,
}

impl Fade {
    pub fn new(length: Duration, sample_rate: usize) -> Self {
        let length_samples = (sample_rate as f64 * length.as_secs_f64()).ceil() as usize;

        let values = (0..length_samples)
            .map(|position| {
                let t = 2.0 * position as f64 / length_samples as f64 - 1.0;
                let value = 0.5 + 0.5 * (t * std::f64::consts::FRAC_PI_2).sin();
                value as f32
            })
            .collect();

        Self { values }
    }

    pub fn bypass() -> Self {
        Self { values: Vec::new() }
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn value(&self, position: usize) -> f32 {
        match self.values.get(position) {
            Some(value) => *value,
            None => 1.0,
        }
    }

    pub fn fade_in_value(&self, position: usize) -> f32 {
        self.value(position)
    }

    pub fn fade_out_value(&self, position: usize) -> f32 {
        self.value(self.len() - position)
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;

    #[test]
    fn starts_and_end() {
        let fade = Fade::new(Duration::from_millis(100), 44100);
        let length = fade.len();
        assert_relative_eq!(fade.value(0), 0.0);
        assert_relative_eq!(fade.value(length / 2), 0.5);
        assert_relative_eq!(fade.value(length - 1), 1.0);
    }

    #[test]
    fn correct_length() {
        let fade = Fade::new(Duration::from_secs(1), 44100);
        assert_eq!(fade.len(), 44100);
    }
}
