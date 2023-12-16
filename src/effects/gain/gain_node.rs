use crate::{
    commands::Id, graph::GraphNode, parameter::*, prelude::*, utility::create_parameters, DspNode,
};

use super::gain_processor::GainProcessor;

/// A node that applies gain to the input
///
/// A gain node must be used in the graph with the same number of input to
/// output channels
///
/// # Parameters
/// - gain
pub struct Gain {
    /// The node to connect into the audio graph
    pub node: GraphNode,

    params: Parameters,
}

const MIN_GAIN: f64 = f64::NEG_INFINITY;
const MAX_GAIN: f64 = f64::INFINITY;
const DEFAULT_GAIN: f64 = 1.0;

impl DspNode for Gain {
    fn get_parameters(&self) -> &Parameters {
        &self.params
    }

    fn get_parameters_mut(&mut self) -> &mut Parameters {
        &mut self.params
    }
}

impl Gain {
    /// Create a new gain node with a given channel count
    pub fn new(context: &dyn Context, channel_count: usize) -> Self {
        let id = Id::generate();

        let (params, realtime_params) = create_parameters(
            id,
            context,
            [(
                "gain",
                ParameterRange::new(DEFAULT_GAIN, MIN_GAIN, MAX_GAIN),
            )],
        );

        let processor = Box::new(GainProcessor::new());

        Self {
            node: GraphNode::new(
                id,
                context,
                channel_count,
                channel_count,
                processor,
                realtime_params,
            ),
            params,
        }
    }

    /// Get the gain parameter
    pub fn gain(&mut self) -> &mut AudioParameter {
        self.get_parameter_mut("gain")
    }
}
