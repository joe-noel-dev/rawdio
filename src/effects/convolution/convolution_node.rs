use crate::{
    commands::Id,
    parameter::{ParameterRange, Parameters},
    utility::create_parameters,
    AudioBuffer, AudioParameter, Context, DspNode, GraphNode, OwnedAudioBuffer,
};

use super::convolution_processor::ConvolutionProcessor;

/// A convolution node to convolve the input signal with an impulse response
///
/// # Parameters
/// - wet
/// - dry
pub struct Convolution {
    /// The node to connect into the audio graph
    pub node: GraphNode,

    params: Parameters,
}

impl DspNode for Convolution {
    fn get_parameters(&self) -> &crate::parameter::Parameters {
        &self.params
    }

    fn get_parameters_mut(&mut self) -> &mut crate::parameter::Parameters {
        &mut self.params
    }
}

impl Convolution {
    /// Create a new convolution node using the given impulse response
    pub fn new(context: &dyn Context, input_count: usize, impulse: OwnedAudioBuffer) -> Self {
        let id = Id::generate();

        let output_count = impulse.channel_count();

        let (params, realtime_params) = create_parameters(
            id,
            context,
            [
                ("wet", ParameterRange::new(1.0, 0.0, 1.0)),
                ("dry", ParameterRange::new(0.0, 0.0, 1.0)),
            ],
        );

        let processor = Box::new(ConvolutionProcessor::new(
            &impulse,
            context.maximum_frame_count(),
            params.get("wet").unwrap().get_id(),
            params.get("dry").unwrap().get_id(),
        ));

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

    /// Get the wet parameter
    pub fn wet(&mut self) -> &mut AudioParameter {
        self.get_parameter_mut("wet")
    }

    /// Get the dry parameter
    pub fn dry(&mut self) -> &mut AudioParameter {
        self.get_parameter_mut("dry")
    }
}
