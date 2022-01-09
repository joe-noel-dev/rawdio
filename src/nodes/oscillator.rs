use std::collections::HashMap;

use lockfree::channel::mpsc::Sender;

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

pub struct OscillatorNode {
    command_queue: Sender<Command>,
    id: Id,
    pub frequency: AudioParameter,
    pub gain: AudioParameter,
}

impl Node for OscillatorNode {
    fn get_id(&self) -> Id {
        self.id
    }

    fn get_command_queue(&self) -> Sender<Command> {
        self.command_queue.clone()
    }
}

const MIN_GAIN: f64 = -2.0;
const MAX_GAIN: f64 = 2.0;
const MIN_FREQUENCY: f64 = 20.0;
const MAX_FREQUENCY: f64 = 20000.0;

impl OscillatorNode {
    pub fn new(command_queue: Sender<Command>, frequency: f64) -> Self {
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
            AudioParameter::new(id, MIN_GAIN, MAX_GAIN, 2.0, command_queue.clone());
        parameters.insert(realtime_gain.get_id(), realtime_gain);

        let dsp = Dsp::new(
            id,
            Box::new(OscillatorDspProcess::new(frequency.get_id(), gain.get_id())),
            parameters,
            1,
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

struct OscillatorDspProcess {
    phase: f64,
    frequency_id: Id,
    gain_id: Id,
}

impl OscillatorDspProcess {
    fn new(frequency_id: Id, gain_id: Id) -> Self {
        Self {
            phase: 0.0,
            frequency_id,
            gain_id,
        }
    }
}

impl DspProcessor for OscillatorDspProcess {
    fn process_audio(
        &mut self,
        output_buffer: &mut dyn AudioBuffer,
        start_time: &Timestamp,
        parameters: &DspParameterMap,
    ) {
        let sample_rate = output_buffer.sample_rate() as f64;

        let frequency = match parameters.get(&self.frequency_id) {
            Some(param) => param,
            None => return,
        };

        let gain = match parameters.get(&self.gain_id) {
            Some(param) => param,
            None => return,
        };

        for frame in 0..output_buffer.num_frames() {
            self.phase += 1.0;

            let frame_time = start_time.incremented_by_samples(frame, sample_rate);
            let frequency = frequency.get_value_at_time(&frame_time);
            let gain = gain.get_value_at_time(&frame_time);

            let value = gain * (std::f64::consts::TAU * frequency * self.phase / sample_rate).sin();

            for channel in 0..output_buffer.num_channels() {
                let location = SampleLocation { channel, frame };
                output_buffer.set_sample(&location, value as f32);
            }
        }
    }
}
