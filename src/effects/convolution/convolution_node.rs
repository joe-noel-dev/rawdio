use crate::{
    commands::Id, graph::DspParameters, AudioBuffer, AudioParameter, Context, GraphNode,
    OwnedAudioBuffer,
};

use super::convolution_processor::ConvolutionProcessor;

/// A convolution node to convolve the input signal with an impulse response
pub struct Convolution {
    /// The node to connect into the audio graph
    pub node: GraphNode,

    /// The proportion of the wet (processed signal) to include in the output
    ///
    /// A value of 1.0 represents a fully wet signal
    /// A value of 0.0 represents a fully dry signal
    pub wet: AudioParameter,
}

impl Convolution {
    /// Create a new convolution node using the given impulse response
    pub fn new(context: &dyn Context, input_count: usize, impulse: OwnedAudioBuffer) -> Self {
        let id = Id::generate();

        let output_count = impulse.channel_count();

        let processor = Box::new(ConvolutionProcessor::new(&impulse));

        let (wet, realtime_wet) =
            AudioParameter::new(Id::generate(), 1.0, 0.0, 1.0, context.get_command_queue());

        let parameters = DspParameters::new([realtime_wet]);

        let node = GraphNode::new(
            id,
            context.get_command_queue(),
            input_count,
            output_count,
            processor,
            parameters,
        );

        Self { node, wet }
    }
}
