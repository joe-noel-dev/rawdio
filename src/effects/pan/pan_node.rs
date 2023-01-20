use std::collections::HashMap;

use crate::{commands::Id, AudioParameter, CommandQueue, Node};

use super::pan_processor::PanProcessor;

const MIN_PAN: f64 = -1.0;
const MAX_PAN: f64 = 1.0;

pub struct PanNode {
    pub node: Node,
    pub pan: AudioParameter,
}

impl PanNode {
    pub fn new(command_queue: CommandQueue, input_count: usize) -> Self {
        let id = Id::generate();

        let (pan, realtime_pan) =
            AudioParameter::new(id, 0.0, MIN_PAN, MAX_PAN, command_queue.clone());

        let parameters = HashMap::from([(realtime_pan.get_id(), realtime_pan)]);

        let output_count = 2;

        let processor = Box::new(PanProcessor::new(pan.get_id()));

        let node = Node::new(
            id,
            command_queue,
            input_count,
            output_count,
            processor,
            parameters,
        );

        Self { node, pan }
    }
}
