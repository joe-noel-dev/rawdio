use crate::{
    commands::Id, graph::DspParameters, AudioBuffer, AudioParameter, Context, GraphNode,
    OwnedAudioBuffer,
};

use super::convolution_processor::ConvolutionProcessor;

pub struct Convolution {
    pub node: GraphNode,
    pub wet: AudioParameter,
}

impl Convolution {
    pub fn new(context: &dyn Context, input_count: usize, impulse: OwnedAudioBuffer) -> Self {
        let id = Id::generate();

        let output_count = impulse.channel_count();

        let processor = Box::new(ConvolutionProcessor::new(impulse));

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
