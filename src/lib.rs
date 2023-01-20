mod buffer;
mod commands;
mod effects;
mod engine;
mod graph;
mod parameter;
mod realtime;
mod utility;

pub type AudioParameter = parameter::AudioParameter;
pub type Timestamp = utility::Timestamp;
pub type Command = commands::Command;
pub type Level = utility::Level;

pub type BorrowedAudioBuffer<'a> = buffer::BorrowedAudioBuffer<'a>;
pub type MutableBorrowedAudioBuffer<'a> = buffer::MutableBorrowedAudioBuffer<'a>;
pub type OwnedAudioBuffer = buffer::OwnedAudioBuffer;
pub type SampleLocation = buffer::SampleLocation;

pub type Adsr = effects::Adsr;
pub type Biquad = effects::Biquad;
pub type BiquadFilterType = effects::BiquadFilterType;
pub type Gain = effects::Gain;
pub type Mixer = effects::Mixer;
pub type Oscillator = effects::Oscillator;
pub type Pan = effects::Pan;
pub type Recorder = effects::Recorder;
pub type Sampler = effects::Sampler;

pub type CommandQueue = lockfree::channel::mpsc::Sender<Command>;

pub use buffer::AudioBuffer;
pub use engine::create_engine;
pub use engine::AudioProcess;
pub use engine::Context;
pub use graph::Node;

#[macro_use]
extern crate lazy_static;
