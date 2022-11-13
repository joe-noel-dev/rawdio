use std::collections::HashMap;

use crate::{
    commands::Id,
    graph::{Dsp, Node},
    CommandQueue, OwnedAudioBuffer, Timestamp,
};

use super::{
    event::SamplerEvent,
    sampler_processor::{EventTransmitter, SamplerDspProcess},
};

pub struct SamplerNode {
    command_queue: CommandQueue,
    id: Id,
    event_transmitter: EventTransmitter,
}

impl Node for SamplerNode {
    fn get_id(&self) -> Id {
        self.id
    }

    fn get_command_queue(&self) -> CommandQueue {
        self.command_queue.clone()
    }
}

impl SamplerNode {
    pub fn new(command_queue: CommandQueue, sample_rate: usize, sample: OwnedAudioBuffer) -> Self {
        let id = Id::generate();

        let (event_transmitter, event_receiver) = lockfree::channel::spsc::create();

        let parameters = HashMap::new();

        let sampler_process = SamplerDspProcess::new(sample_rate, sample, event_receiver);

        let input_count = 0;
        let output_count = 2;

        let dsp = Dsp::new(
            id,
            input_count,
            output_count,
            Box::new(sampler_process),
            parameters,
        );

        dsp.add_to_audio_process(&command_queue);

        Self {
            command_queue,
            id,
            event_transmitter,
        }
    }

    pub fn start_now(&mut self) {
        let _ = self.event_transmitter.send(SamplerEvent::start_now());
    }

    pub fn stop_now(&mut self) {
        let _ = self.event_transmitter.send(SamplerEvent::stop_now());
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

    pub fn enable_loop(&mut self, loop_start: Timestamp, loop_end: Timestamp) {
        let _ = self
            .event_transmitter
            .send(SamplerEvent::enable_loop(loop_start, loop_end));
    }

    pub fn cancel_loop(&mut self) {
        let _ = self.event_transmitter.send(SamplerEvent::cancel_loop());
    }

    pub fn cancel_all(&mut self) {
        let _ = self.event_transmitter.send(SamplerEvent::cancel_all());
    }

    pub fn enable_loop_at_time(
        &mut self,
        enable_at_time: Timestamp,
        loop_start: Timestamp,
        loop_end: Timestamp,
    ) {
        let _ = self
            .event_transmitter
            .send(SamplerEvent::enable_loop_at_time(
                enable_at_time,
                loop_start,
                loop_end,
            ));
    }

    pub fn cancel_loop_at_time(&mut self, cancel_time: Timestamp) {
        let _ = self
            .event_transmitter
            .send(SamplerEvent::cancel_loop_at_time(cancel_time));
    }
}

impl Drop for SamplerNode {
    fn drop(&mut self) {
        Dsp::remove_from_audio_process(self.id, &self.command_queue);
    }
}
