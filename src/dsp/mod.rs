mod adsr;
mod biquad;
mod gain;
mod mixer;
mod oscillator;
mod pan;
mod recorder;
mod sampler;
mod utility;

pub type Gain = gain::GainNode;
pub type Oscillator = oscillator::OscillatorNode;
pub type Pan = pan::PanNode;
pub type Sampler = sampler::SamplerNode;
pub type Adsr = adsr::AdsrNode;
pub type Mixer = mixer::MixerNode;
pub type Recorder = recorder::RecorderNode;
pub type Biquad = biquad::BiquadNode;
pub type BiquadFilterType = biquad::BiquadFilterType;

use lockfree::channel::spsc as Channel;
