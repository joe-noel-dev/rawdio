use super::envelope_notification::{EnvelopeNotification, EnvelopeNotificationTransmitter};
use crate::{effects::utility::*, graph::DspProcessor, prelude::*};
use std::time::Duration;

pub struct EnvelopeProcessor {
    envelopes: Vec<EnvelopeFollower>,
    notification: PeriodicNotification,
    peaks: Vec<f32>,
    transmitter: EnvelopeNotificationTransmitter,
}

impl EnvelopeProcessor {
    pub fn new(
        sample_rate: usize,
        channel_count: usize,
        attack_time: Duration,
        release_time: Duration,
        notification_frequency: f64,
        transmitter: EnvelopeNotificationTransmitter,
    ) -> Self {
        Self {
            envelopes: (0..channel_count)
                .map(|_| EnvelopeFollower::new(sample_rate as f64, attack_time, release_time))
                .collect(),
            notification: PeriodicNotification::new(sample_rate, notification_frequency),
            peaks: (0..channel_count).map(|_| 0.0_f32).collect(),
            transmitter,
        }
    }

    fn send_notifications(&mut self, channel_count: usize) {
        for channel in 0..channel_count {
            let peak = self.peaks.get(channel).expect("Invalid channel index");
            let notification = EnvelopeNotification::new(channel, *peak);
            let _ = self.transmitter.send(notification);
        }

        self.peaks.fill_with(|| 0.0_f32);
    }
}

impl DspProcessor for EnvelopeProcessor {
    fn process_audio(&mut self, context: &mut crate::ProcessContext) {
        let mut position = 0;
        let channel_count = context.input_buffer.channel_count();

        while position < channel_count {
            let frame_count = std::cmp::min(
                context.input_buffer.frame_count(),
                self.notification.samples_until_next_notification(),
            );

            for channel in 0..channel_count {
                let location = SampleLocation::channel(channel);
                let channel_data = context.input_buffer.get_channel_data(location);
                let channel_data = &channel_data[position..position + frame_count];

                let envelope = self
                    .envelopes
                    .get_mut(channel)
                    .expect("Too many input channels");

                let peak = self
                    .peaks
                    .get_mut(channel)
                    .expect("Too many input channels");

                for sample in channel_data {
                    let envelope_value = envelope.process(*sample);
                    *peak = peak.max(envelope_value);
                }
            }

            if self
                .notification
                .advance(context.input_buffer.frame_count())
            {
                self.send_notifications(channel_count);
            }

            position += frame_count;
        }
    }
}
