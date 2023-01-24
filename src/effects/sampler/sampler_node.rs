use crate::{
    commands::Id, effects::Channel, graph::DspParameters, AudioBuffer, CommandQueue, GraphNode,
    OwnedAudioBuffer, Timestamp,
};

use super::{
    sampler_event::SamplerEvent,
    sampler_processor::{EventTransmitter, SamplerDspProcess},
};

pub struct Sampler {
    pub node: GraphNode,
    event_transmitter: EventTransmitter,
}

impl Sampler {
    pub fn new(command_queue: CommandQueue, sample_rate: usize, sample: OwnedAudioBuffer) -> Self {
        let id = Id::generate();

        let (event_transmitter, event_receiver) = Channel::create();

        let input_count = 0;
        let output_count = sample.channel_count();

        let processor = Box::new(SamplerDspProcess::new(sample_rate, sample, event_receiver));

        let node = GraphNode::new(
            id,
            command_queue,
            input_count,
            output_count,
            processor,
            DspParameters::new(),
        );

        Self {
            node,
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
