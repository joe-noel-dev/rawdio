use super::pan_processor::PanProcessor;
use crate::{
    commands::Id,
    graph::DspNode,
    parameter::{ParameterRange, Parameters},
    prelude::*,
    utility::create_parameters,
};

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

        let processor = Box::new(PanProcessor::new());

        Self {
            node: GraphNode::new(
                id,
                context,
                input_count,
                output_count,
                processor,
                realtime_params,
            ),
            params,
        }
    }

    /// Get the pan parameter
    pub fn pan(&mut self) -> &mut AudioParameter {
        self.get_parameter_mut("pan")
    }
}
