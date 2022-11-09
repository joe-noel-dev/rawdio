mod audio_process;
mod buffer;
mod commands;
mod context;
mod dsp;
mod engine;
mod graph;
mod parameter;
mod realtime;
mod timestamp;
mod utility;

pub type AudioParameter = parameter::audio_parameter::AudioParameter;
pub type Command = commands::command::Command;
pub type Gain = dsp::gain::node::GainNode;
pub type Level = utility::level::Level;
pub type Oscillator = dsp::oscillator::node::OscillatorNode;
pub type SampleLocation = buffer::sample_location::SampleLocation;
pub type Sampler = dsp::sampler::node::SamplerNode;
pub type Timestamp = timestamp::Timestamp;
pub type OwnedAudioBuffer = buffer::owned_audio_buffer::OwnedAudioBuffer;
pub type BorrowedAudioBuffer<'a> = buffer::borrowed_audio_buffer::BorrowedAudioBuffer<'a>;
pub type Pan = dsp::pan::node::PanNode;

pub use audio_process::AudioProcess;
pub use buffer::audio_buffer::AudioBuffer;
pub use context::Context;
pub use engine::create_context;
pub use graph::node::Node;

#[macro_use]
extern crate lazy_static;
