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

        while position < context.input_buffer.frame_count() {
            let frame_count = std::cmp::min(
                context.input_buffer.frame_count() - position,
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

            if self.notification.advance(frame_count) {
                self.send_notifications(channel_count);
            }

            position += frame_count;
        }
    }
}

#[cfg(test)]
mod tests {
    use crossbeam::channel;

    use super::*;
    use crate::{graph::DspParameters, ProcessContext};

    #[test]
    fn test_envelope_processor() {
        let sample_rate = 48_000;
        let channel_count = 1;
        let attack_time = Duration::from_millis(0);
        let release_time = Duration::from_millis(100);
        let notification_frequency = 2.0;
        let (tx, rx) = channel::unbounded();

        let mut processor = EnvelopeProcessor::new(
            sample_rate,
            channel_count,
            attack_time,
            release_time,
            notification_frequency,
            tx,
        );

        let frame_count = 45;
        let input = OwnedAudioBuffer::new(frame_count, channel_count, sample_rate);
        let mut output = OwnedAudioBuffer::new(frame_count, channel_count, sample_rate);

        let mut total_frames = 48_000;
        let mut offset = 0;
        while total_frames > 0 {
            let frames = std::cmp::min(frame_count, total_frames);

            processor.process_audio(&mut ProcessContext {
                input_buffer: &input,
                output_buffer: &mut output,
                start_time: &Timestamp::from_samples(offset as f64, sample_rate),
                parameters: &DspParameters::empty(),
            });

            total_frames -= frames;
            offset += frames;
        }

        let event_count = rx.len();
        assert_eq!(event_count, 2);
    }
}
