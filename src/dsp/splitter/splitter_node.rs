use std::collections::HashMap;

use crate::{commands::Id, graph::Dsp, CommandQueue, Node};

use super::splitter_processor::SplitterProcessor;

pub struct SplitterNode {
    command_queue: CommandQueue,
    id: Id,
}

impl SplitterNode {
    pub fn new(command_queue: CommandQueue, input_count: usize, output_count: usize) -> Self {
        let id = Id::generate();
        let parameters = HashMap::new();

        let processor = SplitterProcessor::new();

        let dsp = Dsp::new(
            id,
            input_count,
            output_count,
            Box::new(processor),
            parameters,
        );

        dsp.add_to_audio_process(&command_queue);

        Self { id, command_queue }
    }
}

impl Node for SplitterNode {
    fn get_id(&self) -> crate::commands::Id {
        self.id
    }

    fn get_command_queue(&self) -> crate::CommandQueue {
        self.command_queue.clone()
    }
}
