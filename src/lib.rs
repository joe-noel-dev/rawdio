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
pub type Timestamp = timestamp::Timestamp;
pub type Command = commands::Command;
pub type Level = utility::Level;

pub type BorrowedAudioBuffer<'a> = buffer::BorrowedAudioBuffer<'a>;
pub type MutableBorrowedAudioBuffer<'a> = buffer::MutableBorrowedAudioBuffer<'a>;
pub type OwnedAudioBuffer = buffer::OwnedAudioBuffer;
pub type SampleLocation = buffer::SampleLocation;

pub type Adsr = dsp::Adsr;
pub type Gain = dsp::Gain;
pub type Mixer = dsp::Mixer;
pub type Oscillator = dsp::Oscillator;
pub type Pan = dsp::Pan;
pub type Recorder = dsp::Recorder;
pub type Sampler = dsp::Sampler;
pub type Biquad = dsp::Biquad;
pub type BiquadFilterType = dsp::BiquadFilterType;

pub type CommandQueue = lockfree::channel::mpsc::Sender<Command>;

pub use audio_process::AudioProcess;
pub use buffer::AudioBuffer;
pub use context::Context;
pub use engine::create_engine;
pub use graph::Node;

#[macro_use]
extern crate lazy_static;
