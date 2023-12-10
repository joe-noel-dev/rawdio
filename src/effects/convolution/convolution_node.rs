use crate::{
    commands::Id, graph::DspParameters, parameter::ParameterRange, AudioBuffer, AudioParameter,
    Context, GraphNode, OwnedAudioBuffer,
};

use super::convolution_processor::ConvolutionProcessor;

/// A convolution node to convolve the input signal with an impulse response
pub struct Convolution {
    /// The node to connect into the audio graph
    pub node: GraphNode,

    /// The gain applied to the wet (processed signal) in the output
    pub wet: AudioParameter,

    /// The gain applied to the dry (input signal) in the output
    pub dry: AudioParameter,
}

impl Convolution {
    /// Create a new convolution node using the given impulse response
    pub fn new(context: &dyn Context, input_count: usize, impulse: OwnedAudioBuffer) -> Self {
        let id = Id::generate();

        let output_count = impulse.channel_count();

        let (wet, realtime_wet) =
            AudioParameter::new(id, ParameterRange::new(1.0, 0.0, 1.0), context);

        let (dry, realtime_dry) =
            AudioParameter::new(id, ParameterRange::new(0.0, 0.0, 1.0), context);

        let processor = Box::new(ConvolutionProcessor::new(
            &impulse,
            context.maximum_frame_count(),
            wet.get_id(),
            dry.get_id(),
        ));

        let parameters = DspParameters::new([realtime_wet, realtime_dry]);

        let node = GraphNode::new(
            id,
            context,
            input_count,
            output_count,
            processor,
            parameters,
        );

        Self { node, wet, dry }
    }
}
