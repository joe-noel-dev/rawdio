use std::collections::HashMap;

use lockfree::prelude::mpsc::Sender;

use crate::{
    buffer::{audio_buffer::AudioBuffer, sample_location::SampleLocation},
    commands::{command::Command, id::Id},
    graph::{
        dsp::{Dsp, DspParameterMap, DspProcessor},
        node::Node,
    },
    parameter::AudioParameter,
    timestamp::Timestamp,
};

pub struct GainNode {
    id: Id,
    command_queue: Sender<Command>,
    pub gain: AudioParameter,
}

const MIN_GAIN: f64 = -2.0;
const MAX_GAIN: f64 = 2.0;

impl GainNode {
    pub fn new(command_queue: Sender<Command>) -> Self {
        let mut parameters = HashMap::new();

        let id = Id::generate();

        let (gain, realtime_gain) =
            AudioParameter::new(id, 1.0, MIN_GAIN, MAX_GAIN, command_queue.clone());
        parameters.insert(realtime_gain.get_id(), realtime_gain);

        let dsp = Dsp::new(id, Box::new(GainProcessor::new(gain.get_id())), parameters);

        Dsp::add_to_audio_process(dsp, &command_queue);
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

    fn get_command_queue(&self) -> Sender<Command> {
        self.command_queue.clone()
    }
}

struct GainProcessor {
    gain_id: Id,
}

impl GainProcessor {
    pub fn new(gain_id: Id) -> Self {
        Self { gain_id }
    }
}

impl DspProcessor for GainProcessor {
    fn process_audio(
        &mut self,
        output_buffer: &mut dyn AudioBuffer,
        start_time: &Timestamp,
        parameters: &DspParameterMap,
    ) {
        let sample_rate = output_buffer.sample_rate() as f64;

        let gain = match parameters.get(&self.gain_id) {
            Some(param) => param,
            None => return,
        };

        for frame in 0..output_buffer.num_frames() {
            let frame_time = start_time.incremented_by_samples(frame, sample_rate);
            let gain = gain.get_value_at_time(&frame_time);

            for channel in 0..output_buffer.num_channels() {
                let location = SampleLocation::new(channel, frame);
                let value = output_buffer.get_sample(&location);
                output_buffer.set_sample(&location, value * (gain as f32));
            }
        }
    }
}
