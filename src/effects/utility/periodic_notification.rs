pub struct PeriodicNotification {
    notification_interval: f64,
    next_notification: f64,
}

impl PeriodicNotification {
    pub fn new(sample_rate: usize, notification_frequency: f64) -> Self {
        assert!(sample_rate > 0);
        assert!(notification_frequency > 0.0);

        let notification_interval = sample_rate as f64 / notification_frequency;

        Self {
            notification_interval,
            next_notification: notification_interval,
        }
    }

    pub fn samples_until_next_notification(&self) -> usize {
        self.next_notification.ceil() as usize
    }

    pub fn advance(&mut self, sample_count: usize) -> bool {
        self.next_notification -= sample_count as f64;

        let should_notify = self.next_notification <= 0.0;

        while self.next_notification <= 0.0 {
            self.next_notification += self.notification_interval;
        }

        should_notify
    }
}
