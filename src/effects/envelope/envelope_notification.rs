use crate::effects::Channel;

pub struct EnvelopeNotification {
    channel_index: usize,
    peak_level: f32,
}

impl EnvelopeNotification {
    pub fn new(channel_index: usize, peak_level: f32) -> Self {
        Self {
            channel_index,
            peak_level,
        }
    }

    pub fn channel_index(&self) -> usize {
        self.channel_index
    }

    pub fn peak_level(&self) -> f32 {
        self.peak_level
    }
}

pub type EnvelopeNotificationTransmitter = Channel::Sender<EnvelopeNotification>;
pub type EnvelopeNotificationReceiver = Channel::Receiver<EnvelopeNotification>;
