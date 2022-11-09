use std::collections::HashMap;

use lockfree::channel::mpsc::Sender;

use crate::{commands::id::Id, graph::dsp::Dsp, AudioParameter, Command, Node};

use super::processor::PanProcessor;

const MIN_PAN: f64 = -1.0;
const MAX_PAN: f64 = 1.0;

pub struct PanNode {
    command_queue: Sender<Command>,
    id: Id,
    pub pan: AudioParameter,
}

impl Node for PanNode {
    fn get_id(&self) -> Id {
        self.id
    }

    fn get_command_queue(&self) -> Sender<crate::commands::command::Command> {
        self.command_queue.clone()
    }
}

impl PanNode {
    pub fn new(command_queue: Sender<Command>) -> Self {
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
