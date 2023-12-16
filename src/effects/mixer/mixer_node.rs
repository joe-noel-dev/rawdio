use crate::{commands::Id, effects::Channel, graph::DspParameters, prelude::*};

use super::{
    mixer_event::EventTransmitter, mixer_matrix::MixerMatrix, mixer_processor::MixerProcessor,
};

/// A node that mixes between its input and output channels
///
/// This can be used to change the number of channels in the graph (e.g. mono
///  to stereo)
///
/// You can specify a gain matrix to achieve different up- and down-mixing
/// combinations
///
/// The matrix can only be changed once per audio block, so it is not suitable
/// for fast gain changes. Use a [crate::Gain] for this purpose.
pub struct Mixer {
    /// The node to connect to the audio graph
    pub node: GraphNode,

    /// The matrix for how the inputs will be mixed to the outputs
    pub gain_matrix: MixerMatrix,
    event_transmitter: EventTransmitter,
}

static EVENT_CHANNEL_CAPACITY: usize = 32;

impl Mixer {
    /// Create a new mixer node for a given input to output channel combination
    pub fn new(context: &dyn Context, input_count: usize, output_count: usize) -> Self {
        let id = Id::generate();

        let gain_matrix = MixerMatrix::new(input_count, output_count);

        let (event_transmitter, event_receiver) = Channel::bounded(EVENT_CHANNEL_CAPACITY);

        let processor = Box::new(MixerProcessor::new(event_receiver));

        Self {
            node: GraphNode::new(
                id,
                context,
                input_count,
                output_count,
                processor,
                DspParameters::empty(),
            ),
            gain_matrix,
            event_transmitter,
        }
    }

    /// Set the level for a given input to a given output
    pub fn set_level(&mut self, input_channel: usize, output_channel: usize, level: Level) {
        self.gain_matrix
            .set_level(input_channel, output_channel, level);

        let _ = self.event_transmitter.send(self.gain_matrix.clone());
    }

    /// Create a mixer that converts from mono to stereo
    pub fn mono_to_stereo_splitter(context: &dyn Context) -> Self {
        let input_count = 1;
        let output_count = 2;

        let mut mixer = Self::new(context, input_count, output_count);

        mixer.set_level(0, 0, Level::unity());
        mixer.set_level(0, 1, Level::unity());

        mixer
    }

    /// Create a unity mixer that passes channels through unchanged
    ///
    /// This is useful as an output node to connect to a multi-channel soundcard,
    /// for example
    pub fn unity(context: &dyn Context, channel_count: usize) -> Self {
        let mut mixer = Self::new(context, channel_count, channel_count);

        for channel in 0..channel_count {
            mixer.set_level(channel, channel, Level::unity());
        }

        mixer
    }
}
