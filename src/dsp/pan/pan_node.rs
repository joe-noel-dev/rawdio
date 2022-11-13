use std::collections::HashMap;

use crate::{commands::Id, graph::Dsp, AudioParameter, CommandQueue, Node};

use super::pan_processor::PanProcessor;

const MIN_PAN: f64 = -1.0;
const MAX_PAN: f64 = 1.0;

pub struct PanNode {
    command_queue: CommandQueue,
    id: Id,
    pub pan: AudioParameter,
}

impl Node for PanNode {
    fn get_id(&self) -> Id {
        self.id
    }

    fn get_command_queue(&self) -> CommandQueue {
        self.command_queue.clone()
    }
}

impl PanNode {
    pub fn new(command_queue: CommandQueue, input_count: usize) -> Self {
        let id = Id::generate();
        let mut parameters = HashMap::new();

        let (pan, realtime_pan) =
            AudioParameter::new(id, 0.0, MIN_PAN, MAX_PAN, command_queue.clone());

        parameters.insert(realtime_pan.get_id(), realtime_pan);

        let output_count = 2;
        let dsp = Dsp::new(
            id,
            input_count,
            output_count,
            Box::new(PanProcessor::new(pan.get_id())),
            parameters,
        );

        dsp.add_to_audio_process(&command_queue);

        Self {
            command_queue,
            id,
            pan,
        }
    }
}

impl Drop for PanNode {
    fn drop(&mut self) {
        Dsp::remove_from_audio_process(self.id, &self.command_queue)
    }
}
