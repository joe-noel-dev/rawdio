use std::collections::HashMap;

use lockfree::channel::mpsc::Sender;

use crate::{
    commands::{command::Command, id::Id},
    graph::{
        dsp::{Dsp, DspParameterMap, DspProcessFn},
        node::Node,
    },
    parameter::AudioParameter,
    timestamp::Timestamp,
    utility::audio_buffer::{AudioBuffer, SampleLocation},
};

pub struct Oscillator {
    command_queue: Sender<Command>,
    id: Id,
    pub frequency: AudioParameter,
    pub gain: AudioParameter,
}

impl Node for Oscillator {
    fn get_id(&self) -> Id {
        self.id
    }
}

impl Oscillator {
    pub fn new(command_queue: Sender<Command>, frequency: f64) -> Self {
        let id = Id::generate();

        let mut parameters = HashMap::new();
        let (frequency, realtime_frequency) =
            AudioParameter::new(id, frequency, command_queue.clone());
        parameters.insert(realtime_frequency.get_id(), realtime_frequency);

        let (gain, realtime_gain) = AudioParameter::new(id, 1.0, command_queue.clone());
        parameters.insert(realtime_gain.get_id(), realtime_gain);

        let audio_process = Self::process_oscillator(frequency.get_id(), gain.get_id());
        let dsp = Dsp::new(id, audio_process, parameters);
        let _ = command_queue.send(Command::AddDsp(Box::new(dsp)));

        Self {
            command_queue,
            id,
            frequency,
            gain,
        }
    }

    fn process_oscillator(frequency_id: Id, gain_id: Id) -> DspProcessFn {
        let mut phase = 0.0;

        Box::new(
            move |output: &mut dyn AudioBuffer,
                  start_time: &Timestamp,
                  parameters: &DspParameterMap| {
                let sample_rate = output.sample_rate() as f64;

                let frequency = match parameters.get(&frequency_id) {
                    Some(param) => param,
                    None => return,
                };

                let gain = match parameters.get(&gain_id) {
                    Some(param) => param,
                    None => return,
                };

                for frame in 0..output.num_frames() {
                    phase += 1.0;

                    let frame_time = start_time.incremented_by_samples(frame, sample_rate);
                    let frequency = frequency.get_value_at_time(&frame_time);
                    let gain = gain.get_value_at_time(&frame_time);

                    let value =
                        gain * (std::f64::consts::TAU * frequency * phase / sample_rate).sin();
                    for channel in 0..output.num_channels() {
                        let location = SampleLocation { channel, frame };
                        output.set_sample(&location, value as f32);
                    }
                }
            },
        )
    }
}

impl Drop for Oscillator {
    fn drop(&mut self) {
        let _ = self.command_queue.send(Command::RemoveDsp(self.id));
    }
}
