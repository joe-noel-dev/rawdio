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

pub type AudioParameter = parameter::AudioParameter;
pub type BorrowedAudioBuffer<'a> = buffer::BorrowedAudioBuffer<'a>;
pub type Command = commands::Command;
pub type Gain = dsp::Gain;
pub type Level = utility::Level;
pub type Oscillator = dsp::Oscillator;
pub type OwnedAudioBuffer = buffer::OwnedAudioBuffer;
pub type Pan = dsp::Pan;
pub type SampleLocation = buffer::SampleLocation;
pub type Sampler = dsp::Sampler;
pub type Timestamp = timestamp::Timestamp;

pub use audio_process::AudioProcess;
pub use buffer::AudioBuffer;
pub use context::Context;
pub use engine::create_context;
pub use graph::node::Node;

#[macro_use]
extern crate lazy_static;
