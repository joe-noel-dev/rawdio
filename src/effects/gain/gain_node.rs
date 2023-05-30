use crate::{
    commands::Id,
    graph::{DspParameters, GraphNode},
    parameter::{AudioParameter, ParameterRange},
    Context,
};

use super::gain_processor::GainProcessor;

/// A node that applies gain to the input
///
/// A gain node must be used in the graph with the same number of input to
/// output channels
pub struct Gain {
    /// The node to connect into the audio graph
    pub node: GraphNode,

    /// The (linear) value of the gain
    ///
    /// See [crate::Level] to set the value in dB
    pub gain: AudioParameter,
}

const MIN_GAIN: f64 = f64::NEG_INFINITY;
const MAX_GAIN: f64 = f64::INFINITY;
const DEFAULT_GAIN: f64 = 1.0;

impl Gain {
    /// Create a new gain node with a given channel count
    pub fn new(context: &dyn Context, channel_count: usize) -> Self {
        let id = Id::generate();
        let (gain, realtime_gain) = AudioParameter::new(
            id,
            ParameterRange::new(DEFAULT_GAIN, MIN_GAIN, MAX_GAIN),
            context,
        );

        let processor = Box::new(GainProcessor::new(gain.get_id()));

        Self {
            node: GraphNode::new(
                id,
                context,
                channel_count,
                channel_count,
                processor,
                DspParameters::new([realtime_gain]),
            ),
            gain,
        }
    }
}
