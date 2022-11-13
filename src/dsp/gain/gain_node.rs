use std::collections::HashMap;

use crate::{
    commands::Id,
    graph::{Dsp, Node},
    parameter::AudioParameter,
    CommandQueue,
};

use super::gain_processor::GainProcessor;

pub struct GainNode {
    id: Id,
    command_queue: CommandQueue,
    pub gain: AudioParameter,
}

const MIN_GAIN: f64 = f64::NEG_INFINITY;
const MAX_GAIN: f64 = f64::INFINITY;
const DEFAULT_GAIN: f64 = 1.0;

impl GainNode {
    pub fn new(command_queue: CommandQueue, channel_count: usize) -> Self {
        let mut parameters = HashMap::new();

        let id = Id::generate();

        let (gain, realtime_gain) =
            AudioParameter::new(id, DEFAULT_GAIN, MIN_GAIN, MAX_GAIN, command_queue.clone());
        parameters.insert(realtime_gain.get_id(), realtime_gain);

        let dsp = Dsp::new(
            id,
            channel_count,
            channel_count,
            Box::new(GainProcessor::new(gain.get_id())),
            parameters,
        );

        dsp.add_to_audio_process(&command_queue);

        Self {
            id,
            command_queue,
            gain,
        }
    }
}

impl Node for GainNode {
    fn get_id(&self) -> Id {
        self.id
    }

    fn get_command_queue(&self) -> CommandQueue {
        self.command_queue.clone()
    }
}
