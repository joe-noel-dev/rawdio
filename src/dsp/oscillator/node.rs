use std::collections::HashMap;

use crate::{
    commands::id::Id,
    graph::{
        dsp::Dsp,
        node::{CommandQueue, Node},
    },
    parameter::audio_parameter::AudioParameter,
};

use super::processor::OscillatorDspProcess;

pub struct OscillatorNode {
    command_queue: CommandQueue,
    id: Id,
    pub frequency: AudioParameter,
    pub gain: AudioParameter,
}

impl Node for OscillatorNode {
    fn get_id(&self) -> Id {
        self.id
    }

    fn get_command_queue(&self) -> CommandQueue {
        self.command_queue.clone()
    }
}

const MIN_GAIN: f64 = -2.0;
const MAX_GAIN: f64 = 2.0;
const MIN_FREQUENCY: f64 = 20.0;
const MAX_FREQUENCY: f64 = 20000.0;

impl OscillatorNode {
    pub fn new(command_queue: CommandQueue, frequency: f64) -> Self {
        let id = Id::generate();

        let mut parameters = HashMap::new();
        let (frequency, realtime_frequency) = AudioParameter::new(
            id,
            frequency,
            MIN_FREQUENCY,
            MAX_FREQUENCY,
            command_queue.clone(),
        );
        parameters.insert(realtime_frequency.get_id(), realtime_frequency);

        let (gain, realtime_gain) =
            AudioParameter::new(id, 1.0, MIN_GAIN, MAX_GAIN, command_queue.clone());
        parameters.insert(realtime_gain.get_id(), realtime_gain);

        let dsp = Dsp::new(
            id,
            Box::new(OscillatorDspProcess::new(frequency.get_id(), gain.get_id())),
            parameters,
        );

        Dsp::add_to_audio_process(dsp, &command_queue);

        Self {
            command_queue,
            id,
            frequency,
            gain,
        }
    }
}

impl Drop for OscillatorNode {
    fn drop(&mut self) {
        Dsp::remove_from_audio_process(self.id, &self.command_queue);
    }
}
