mod adsr;
mod gain;
mod mixer;
mod oscillator;
mod pan;
mod sampler;

pub type Gain = gain::GainNode;
pub type Oscillator = oscillator::OscillatorNode;
pub type Pan = pan::PanNode;
pub type Sampler = sampler::SamplerNode;
pub type Adsr = adsr::AdsrNode;
pub type Mixer = mixer::MixerNode;

use lockfree::channel::spsc as Channel;
