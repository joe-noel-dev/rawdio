use std::collections::HashMap;

use lockfree::channel::mpsc::Sender;

use crate::{
    commands::{command::Command, id::Id},
    graph::{dsp::Dsp, node::Node},
    OwnedAudioBuffer,
};

use super::processor::SamplerDspProcess;

pub struct SamplerNode {
    command_queue: Sender<Command>,
    id: Id,
}

impl Node for SamplerNode {
    fn get_id(&self) -> Id {
        self.id
    }

    fn get_command_queue(&self) -> Sender<Command> {
        self.command_queue.clone()
    }
}

impl SamplerNode {
    pub fn new(
        command_queue: Sender<Command>,
        sample_rate: usize,
        sample: OwnedAudioBuffer,
    ) -> Self {
        let id = Id::generate();

        let parameters = HashMap::new();

        let mut sampler_process = SamplerDspProcess::new(sample_rate, sample);
        sampler_process.start(0);

        let dsp = Dsp::new(id, Box::new(sampler_process), parameters);

        Dsp::add_to_audio_process(dsp, &command_queue);

        Self { command_queue, id }
    }
}

impl Drop for SamplerNode {
    fn drop(&mut self) {
        Dsp::remove_from_audio_process(self.id, &self.command_queue);
    }
}
