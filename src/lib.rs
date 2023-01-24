mod buffer;
mod commands;
mod effects;
mod engine;
mod graph;
mod parameter;
mod realtime;
mod utility;

pub(crate) use commands::Command;
pub(crate) use parameter::AudioParameter;

pub use utility::Level;
pub use utility::Timestamp;

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

pub type CommandQueue = lockfree::channel::mpsc::Sender<Command>;

pub use buffer::AudioBuffer;
pub use engine::create_engine;
pub use engine::AudioProcess;
pub use engine::Context;
pub use graph::GraphNode;

#[macro_use]
extern crate lazy_static;
