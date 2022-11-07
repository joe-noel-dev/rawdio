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

pub type AudioBufferSlice<'a> = buffer::audio_buffer_slice::AudioBufferSlice<'a>;
pub type AudioParameter = parameter::audio_parameter::AudioParameter;
pub type BorrowedAudioBuffer<'a> = buffer::borrowed_audio_buffer::BorrowedAudioBuffer<'a>;
pub type Command = commands::command::Command;
pub type Gain = dsp::gain::node::GainNode;
pub type ImmutableAudioBufferSlice<'a> =
    buffer::immutable_audio_buffer_slice::ImmutableAudioBufferSlice<'a>;
pub type Level = utility::level::Level;
pub type Oscillator = dsp::oscillator::node::OscillatorNode;
pub type OwnedAudioBuffer = buffer::owned_audio_buffer::OwnedAudioBuffer;
pub type SampleLocation = buffer::sample_location::SampleLocation;
pub type Sampler = dsp::sampler::node::SamplerNode;
pub type Timestamp = timestamp::Timestamp;

pub use audio_process::AudioProcess;
pub use buffer::audio_buffer::AudioBuffer;
pub use context::Context;
pub use engine::create_context;
pub use graph::node::Node;

#[macro_use]
extern crate lazy_static;
