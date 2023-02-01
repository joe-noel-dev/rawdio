#![feature(portable_simd)]

mod buffer;
mod commands;
mod effects;
mod engine;
mod graph;
mod parameter;
mod realtime;
mod utility;

pub(crate) use commands::Command;

pub use buffer::AudioBuffer;
pub use buffer::BorrowedAudioBuffer;
pub use buffer::MutableBorrowedAudioBuffer;
pub use buffer::OwnedAudioBuffer;
pub use buffer::SampleLocation;

pub use effects::Adsr;
pub use effects::Biquad;
pub use effects::BiquadFilterType;
pub use effects::Envelope;
pub use effects::Gain;
pub use effects::Mixer;
pub use effects::Oscillator;
pub use effects::Pan;
pub use effects::Recorder;
pub use effects::Sampler;
pub use effects::Waveshaper;

pub use engine::create_engine;
pub use engine::AudioProcess;
pub use engine::CommandQueue;
pub use engine::Context;

pub use graph::GraphNode;

pub use parameter::AudioParameter;

pub use utility::Level;
pub use utility::Timestamp;

pub(crate) const MAXIMUM_FRAME_COUNT: usize = 512;
pub(crate) const MAXIMUM_CHANNEL_COUNT: usize = 2;
