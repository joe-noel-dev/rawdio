use std::sync::atomic::Ordering;

use lockfree::channel::mpsc::Sender;

use crate::{
    commands::{command::Command, id::Id},
    graph::{dsp::Dsp, node::Node},
    parameter::{AudioParameter, ParameterValue},
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
    pub fn new(command_queue: Sender<Command>, frequency: f32) -> Self {
        let id = Id::generate();

        let frequency = AudioParameter::new(frequency, command_queue.clone());
        let gain = AudioParameter::new(1.0, command_queue.clone());

        let dsp = Self::create_dsp(id, frequency.get_value(), gain.get_value());
        let _ = command_queue.send(Command::AddDsp(Box::new(dsp)));

        Self {
            command_queue,
            id,
            frequency,
            gain,
        }
    }

    fn create_dsp(id: Id, frequency: ParameterValue, gain: ParameterValue) -> Dsp {
        Dsp::new(id, Box::new(Self::process_oscillator(frequency, gain)))
    }

    fn process_oscillator(
        frequency: ParameterValue,
        gain: ParameterValue,
    ) -> impl FnMut(&mut dyn AudioBuffer) {
        let mut phase = 0.0f32;

        move |output: &mut dyn AudioBuffer| {
            let sample_rate = output.sample_rate() as f32;
            let frequency = frequency.load(Ordering::Acquire);
            let gain = gain.load(Ordering::Acquire);

            for frame in 0..output.num_frames() {
                phase += 1.0;

                let value = gain * (std::f32::consts::TAU * frequency * phase / sample_rate).sin();
                for channel in 0..output.num_channels() {
                    let location = SampleLocation { channel, frame };
                    output.set_sample(&location, value);
                }
            }
        }
    }
}

impl Drop for Oscillator {
    fn drop(&mut self) {
        let _ = self.command_queue.send(Command::RemoveDsp(self.id));
    }
}
