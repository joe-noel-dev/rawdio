use crate::{commands::Id, graph::DspParameters, CommandQueue, Node};

use super::splitter_processor::SplitterProcessor;

pub struct SplitterNode {
    pub node: Node,
}

impl SplitterNode {
    pub fn new(command_queue: CommandQueue, input_count: usize, output_count: usize) -> Self {
        let id = Id::generate();

        let processor = Box::new(SplitterProcessor::new());

        let node = Node::new(
            id,
            command_queue,
            input_count,
            output_count,
            processor,
            DspParameters::new(),
        );

        Self { node }
    }
}
