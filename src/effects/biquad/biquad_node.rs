use crate::{
    commands::Id,
    parameter::{ParameterRange, Parameters},
    utility::create_parameters,
    Context, DspNode, GraphNode, Level,
};

use super::{biquad_processor::BiquadProcessor, filter_type::BiquadFilterType};

/// A biquad filter
///
/// This can be used to create a second order filter
///
/// # Parameters
/// - frequency
/// - q
/// - shelf-gain
/// - gain
pub struct Biquad {
    /// The node to connect into the audio graph
    pub node: GraphNode,

    parameters: Parameters,
}

impl DspNode for Biquad {
    fn get_parameters(&self) -> &crate::parameter::Parameters {
        &self.parameters
    }

    fn get_parameters_mut(&mut self) -> &mut crate::parameter::Parameters {
        &mut self.parameters
    }
}

impl Biquad {
    /// Create a new filter of a type
    pub fn new(context: &dyn Context, channel_count: usize, filter_type: BiquadFilterType) -> Self {
        let id = Id::generate();

        let (params, realtime_params) = create_parameters(
            id,
            context,
            [
                ("frequency", ParameterRange::new(1_000.0, 20.0, 20_000.0)),
                ("q", ParameterRange::new(1.0 / 2.0_f64.sqrt(), 0.1, 10.0)),
                (
                    "shelf-gain",
                    ParameterRange::new(
                        Level::unity().as_linear(),
                        0.0,
                        Level::from_db(100.0).as_linear(),
                    ),
                ),
                (
                    "gain",
                    ParameterRange::new(
                        Level::unity().as_linear(),
                        0.0,
                        Level::from_db(100.0).as_linear(),
                    ),
                ),
            ],
        );

        let processor = Box::new(BiquadProcessor::new(
            context.get_sample_rate(),
            channel_count,
            filter_type,
            params.get("frequency").unwrap().get_id(),
            params.get("q").unwrap().get_id(),
            params.get("shelf-gain").unwrap().get_id(),
            params.get("gain").unwrap().get_id(),
        ));

        let node = GraphNode::new(
            id,
            context,
            channel_count,
            channel_count,
            processor,
            realtime_params,
        );

        Self {
            node,
            parameters: params,
        }
    }
}
