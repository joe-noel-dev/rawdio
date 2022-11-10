use std::collections::HashMap;

use crate::{
    commands::Id,
    graph::{dsp::Dsp, node::CommandQueue},
    AudioParameter, Node,
};

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
    pub fn new(command_queue: CommandQueue) -> Self {
        let id = Id::generate();
        let mut parameters = HashMap::new();

        let (pan, realtime_pan) =
            AudioParameter::new(id, 0.0, MIN_PAN, MAX_PAN, command_queue.clone());

        parameters.insert(realtime_pan.get_id(), realtime_pan);

        let dsp = Dsp::new(id, Box::new(PanProcessor::new(pan.get_id())), parameters);

        Dsp::add_to_audio_process(dsp, &command_queue);

        Self {
            command_queue,
            id,
            pan,
        }
    }
}
