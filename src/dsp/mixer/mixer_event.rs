use super::mixer_matrix::MixerMatrix;

pub type EventTransmitter = crate::dsp::Channel::Sender<MixerMatrix>;
pub type EventReceiver = crate::dsp::Channel::Receiver<MixerMatrix>;
