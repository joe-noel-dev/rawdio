use super::mixer_matrix::MixerMatrix;

pub type EventTransmitter = crate::effects::Channel::Sender<MixerMatrix>;
pub type EventReceiver = crate::effects::Channel::Receiver<MixerMatrix>;
