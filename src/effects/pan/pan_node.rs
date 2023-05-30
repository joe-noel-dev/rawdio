use crate::{
    commands::Id, graph::DspParameters, parameter::ParameterRange, AudioParameter, Context,
    GraphNode,
};

use super::pan_processor::PanProcessor;

const MIN_PAN: f64 = -1.0;
const MAX_PAN: f64 = 1.0;

/// A node that will pan the input signal between two output channels
pub struct Pan {
    /// The node to connect to the audio graph
    pub node: GraphNode,

    /// The pan control, where -1.0 represents fully left and 1.0 represents
    /// full right
    pub pan: AudioParameter,
}

impl Pan {
    /// Create a new pan node
    pub fn new(context: &dyn Context, input_count: usize) -> Self {
        let id = Id::generate();

        let (pan, realtime_pan) =
            AudioParameter::new(id, ParameterRange::new(0.0, MIN_PAN, MAX_PAN), context);

        let output_count = 2;

        let processor = Box::new(PanProcessor::new(pan.get_id()));

        let node = GraphNode::new(
            id,
            context,
            input_count,
            output_count,
            processor,
            DspParameters::new([realtime_pan]),
        );

        Self { node, pan }
    }
}
