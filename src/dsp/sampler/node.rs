use std::collections::HashMap;

use lockfree::channel::mpsc::Sender;

use crate::{
    commands::{command::Command, id::Id},
    graph::{dsp::Dsp, node::Node},
    OwnedAudioBuffer, Timestamp,
};

use super::processor::{EventTransmitter, SamplerDspProcess, SamplerEvent};

pub struct SamplerNode {
    command_queue: Sender<Command>,
    id: Id,
    event_transmitter: EventTransmitter,
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

        let (event_transmitter, event_receiver) = lockfree::channel::spsc::create();

        let parameters = HashMap::new();

        let sampler_process = SamplerDspProcess::new(sample_rate, sample, event_receiver);

        let dsp = Dsp::new(id, Box::new(sampler_process), parameters);

        Dsp::add_to_audio_process(dsp, &command_queue);

        Self {
            command_queue,
            id,
            event_transmitter,
        }
    }

    pub fn start_from_position_at_time(
        &mut self,
        start_time: Timestamp,
        position_in_sample: Timestamp,
    ) {
        let _ = self
            .event_transmitter
            .send(SamplerEvent::start(start_time, position_in_sample));
    }

    pub fn stop_at_time(&mut self, stop_time: Timestamp) {
        let _ = self.event_transmitter.send(SamplerEvent::stop(stop_time));
    }
}

impl Drop for SamplerNode {
    fn drop(&mut self) {
        Dsp::remove_from_audio_process(self.id, &self.command_queue);
    }
}
