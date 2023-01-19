use crate::{dsp::Channel, OwnedAudioBuffer};

pub enum RecorderNotification {
    Start,
    Data(OwnedAudioBuffer, usize),
    Stop,
}

pub type RecorderNotificationTransmitter = Channel::Sender<RecorderNotification>;
pub type RecorderNotificationReceiver = Channel::Receiver<RecorderNotification>;
