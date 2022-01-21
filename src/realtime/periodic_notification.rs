pub struct PeriodicNotification {
    interval_samples: i64,
    countdown: i64,
}

impl PeriodicNotification {
    pub fn new(sample_rate: usize, interval_hz: f64) -> Self {
        let interval = (sample_rate as f64 / interval_hz) as i64;
        Self {
            interval_samples: interval,
            countdown: interval,
        }
    }

    pub fn increment(&mut self, num_samples: usize) -> bool {
        let mut should_notify = false;

        self.countdown -= num_samples as i64;

        if self.countdown <= 0 {
            should_notify = true;
        }

        while self.countdown <= 0 {
            self.countdown += self.interval_samples;
        }

        should_notify
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn increments_at_the_right_interval() {
        let mut notification = PeriodicNotification::new(44100, 60.0);
        assert!(!notification.increment(734));
        assert!(notification.increment(1));
        assert!(!notification.increment(734));
        assert!(notification.increment(1));
    }
}
