#![feature(portable_simd)]
#![warn(missing_docs)]

//! rawdio is an audio engine that is inspired by the Web Audio API
//!  
//! You can use it to:
//! - Create a context
//! - Create DSP nodes
//! - Connect the nodes together
//! - Accurately schedule 'events'
//! - Process the graph with an input and an output
//!
//! # Example
//!
//! ```rust
//! use rawdio::{create_engine, AudioProcess, Context, Oscillator, OwnedAudioBuffer};
//!
//! let sample_rate = 48_000;
//! let (mut context, mut process) = create_engine(sample_rate);
//!
//! let frequency = 1_000.0;
//! let channel_count = 2;
//! let oscillator = Oscillator::sine(context.as_ref(), frequency, channel_count);
//!
//! oscillator.node.connect_to_output();
//!
//! context.start();
//!
//! // Call `process.process(...)`, passing in the input samples, and using the output
//! // If you wish to use with your sound card, you could use something like cpal (see the examples)
//! ```

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
pub use effects::Convolution;
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
