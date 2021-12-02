use crate::timestamp::Timestamp;

pub struct Context {
    sample_rate: usize,
    timestamp: Timestamp,
}

impl Context {
    pub fn new(sample_rate: usize) -> Self {
        Self {
            sample_rate,
            timestamp: Timestamp::default(),
        }
    }

    pub fn current_time(&self) -> Timestamp {
        self.timestamp
    }

    pub fn process(&mut self, num_samples: usize) {
        self.timestamp = self.timestamp.incremented(num_samples, self.sample_rate);
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;
    #[test]
    fn it_increments_the_current_time_when_processed() {
        let sample_rate = 44100;
        let mut context = Context::new(sample_rate);
        let timestamp_before = context.current_time();
        context.process(sample_rate);
        assert_relative_eq!(
            context.current_time().get_seconds() - timestamp_before.get_seconds(),
            1.0
        );
    }
}
