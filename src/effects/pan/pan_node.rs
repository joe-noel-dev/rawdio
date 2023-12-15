use crate::{
    commands::Id,
    parameter::{ParameterRange, Parameters},
    utility::create_parameters,
    AudioParameter, Context, DspNode, GraphNode,
};

use super::pan_processor::PanProcessor;

const MIN_PAN: f64 = -1.0;
const MAX_PAN: f64 = 1.0;

/// A node that will pan the input signal between two output channels
///
/// # Parameters
/// - pan
pub struct Pan {
    /// The node to connect to the audio graph
    pub node: GraphNode,

    params: Parameters,
}

impl DspNode for Pan {
    fn get_parameters(&self) -> &crate::parameter::Parameters {
        &self.params
    }

    fn get_parameters_mut(&mut self) -> &mut crate::parameter::Parameters {
        &mut self.params
    }
}

impl Pan {
    /// Create a new pan node
    pub fn new(context: &dyn Context, input_count: usize) -> Self {
        let id = Id::generate();

        let (params, realtime_params) = create_parameters(
            id,
            context,
            [("pan", ParameterRange::new(0.0, MIN_PAN, MAX_PAN))],
        );

        let output_count = 2;

        let processor = Box::new(PanProcessor::new(params.get("pan").unwrap().get_id()));

        let node = GraphNode::new(
            id,
            context,
            input_count,
            output_count,
            processor,
            realtime_params,
        );

        Self { node, params }
    }

    /// Get the pan parameter
    pub fn pan(&mut self) -> &mut AudioParameter {
        self.get_parameter_mut("pan")
    }
}
