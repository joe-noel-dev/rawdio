use std::collections::HashMap;

use crate::{commands::Id, graph::GraphNode, parameter::AudioParameter, CommandQueue};

use super::gain_processor::GainProcessor;

pub struct GainNode {
    pub node: GraphNode,
    pub gain: AudioParameter,
}

const MIN_GAIN: f64 = f64::NEG_INFINITY;
const MAX_GAIN: f64 = f64::INFINITY;
const DEFAULT_GAIN: f64 = 1.0;

impl GainNode {
    pub fn new(command_queue: CommandQueue, channel_count: usize) -> Self {
        let id = Id::generate();
        let (gain, realtime_gain) =
            AudioParameter::new(id, DEFAULT_GAIN, MIN_GAIN, MAX_GAIN, command_queue.clone());

        let parameters = HashMap::from([(realtime_gain.get_id(), realtime_gain)]);
        let processor = Box::new(GainProcessor::new(gain.get_id()));

        Self {
            node: GraphNode::new(
                id,
                command_queue,
                channel_count,
                channel_count,
                processor,
                parameters,
            ),
            gain,
        }
    }
}
