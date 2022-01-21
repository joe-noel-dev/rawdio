mod audio_process;
mod buffer;
mod commands;
mod context;
mod graph;
mod nodes;
mod parameter;
mod realtime;
mod timestamp;
mod utility;

pub type Level = utility::level::Level;
pub type Context = context::Context;
pub type Timestamp = timestamp::Timestamp;

pub type Gain = nodes::gain::GainNode;
pub type Oscillator = nodes::oscillator::OscillatorNode;

pub type AudioBufferSlice<'a> = buffer::audio_buffer_slice::AudioBufferSlice<'a>;
pub type OwnedAudioBuffer = buffer::owned_audio_buffer::OwnedAudioBuffer;
pub type BorrowedAudioBuffer<'a> = buffer::borrowed_audio_buffer::BorrowedAudioBuffer<'a>;

pub type SampleLocation = buffer::sample_location::SampleLocation;

pub use audio_process::AudioProcess;
pub use buffer::audio_buffer::AudioBuffer;
pub use graph::node::Node;

#[macro_use]
extern crate lazy_static;
