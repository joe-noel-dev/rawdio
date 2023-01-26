use crate::{commands::Id, graph::DspParameters, AudioParameter, Context, GraphNode};

use super::pan_processor::PanProcessor;

const MIN_PAN: f64 = -1.0;
const MAX_PAN: f64 = 1.0;

pub struct Pan {
    pub node: GraphNode,
    pub pan: AudioParameter,
}

impl Pan {
    pub fn new(context: &dyn Context, input_count: usize) -> Self {
        let id = Id::generate();

        let (pan, realtime_pan) =
            AudioParameter::new(id, 0.0, MIN_PAN, MAX_PAN, context.get_command_queue());

        let output_count = 2;

        let processor = Box::new(PanProcessor::new(pan.get_id()));

        let node = GraphNode::new(
            id,
            context.get_command_queue(),
            input_count,
            output_count,
            processor,
            DspParameters::new([realtime_pan]),
        );

        Self { node, pan }
    }
}
