mod adsr;
mod gain;
mod oscillator;
mod pan;
mod sampler;
mod splitter;

pub type Gain = gain::GainNode;
pub type Oscillator = oscillator::OscillatorNode;
pub type Pan = pan::PanNode;
pub type Sampler = sampler::SamplerNode;
pub type Splitter = splitter::SplitterNode;
pub type Adsr = adsr::AdsrNode;

use lockfree::channel::spsc as Channel;
