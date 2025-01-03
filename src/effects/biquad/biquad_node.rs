use super::{biquad_processor::BiquadProcessor, filter_type::BiquadFilterType};
use crate::{commands::Id, graph::DspNode, parameter::*, prelude::*, utility::create_parameters};

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
        ));

        Self {
            node: GraphNode::new(
                id,
                context,
                channel_count,
                channel_count,
                processor,
                realtime_params,
            ),
            parameters: params,
        }
    }

    /// Get the frequency parameter
    pub fn frequency(&mut self) -> &mut AudioParameter {
        self.get_parameter_mut("frequency")
    }

    /// Get the Q parameter
    pub fn q(&mut self) -> &mut AudioParameter {
        self.get_parameter_mut("q")
    }

    /// Get the shelf gain parameter
    pub fn shelf_gain(&mut self) -> &mut AudioParameter {
        self.get_parameter_mut("shelf_gain")
    }

    /// Get the gain parameter
    pub fn gain(&mut self) -> &mut AudioParameter {
        self.get_parameter_mut("gain")
    }
}
