use crate::{
    commands::Id,
    graph::{DspParameters, GraphNode},
    parameter::AudioParameter,
    Context,
};

use super::gain_processor::GainProcessor;

pub struct Gain {
    pub node: GraphNode,
    pub gain: AudioParameter,
}

const MIN_GAIN: f64 = f64::NEG_INFINITY;
const MAX_GAIN: f64 = f64::INFINITY;
const DEFAULT_GAIN: f64 = 1.0;

impl Gain {
    pub fn new(context: &dyn Context, channel_count: usize) -> Self {
        let id = Id::generate();
        let (gain, realtime_gain) = AudioParameter::new(
            id,
            DEFAULT_GAIN,
            MIN_GAIN,
            MAX_GAIN,
            context.get_command_queue(),
        );

        let processor = Box::new(GainProcessor::new(gain.get_id()));

        Self {
            node: GraphNode::new(
                id,
                context.get_command_queue(),
                channel_count,
                channel_count,
                processor,
                DspParameters::new([realtime_gain]),
            ),
            gain,
        }
    }
}
