use lockfree::channel::mpsc::Sender;

use crate::{
    commands::{command::Command, id::Id},
    graph::{dsp::Dsp, node::Node},
    parameter::{self, AudioParameter, ParameterValue, RealtimeAudioParameter},
    utility::audio_buffer::{AudioBuffer, SampleLocation},
};

#[derive(Clone)]
pub struct Oscillator {
    command_queue: Sender<Command>,
    id: Id,
    pub frequency: AudioParameter,
}

impl Node for Oscillator {
    fn get_id(&self) -> Id {
        self.id
    }
}

impl Oscillator {
    pub fn new(command_queue: Sender<Command>, frequency: f32) -> Self {
        let id = Id::generate();

        let (frequency, realtime_frequency) =
            parameter::create(ParameterValue::Float(frequency), command_queue.clone());

        let dsp = Self::create_dsp(id, realtime_frequency);
        let _ = command_queue.send(Command::AddDsp(dsp));

        Self {
            command_queue,
            id,
            frequency,
        }
    }

    pub fn remove(&mut self) {
        let _ = self.command_queue.send(Command::RemoveDsp(self.id));
    }

    pub fn create_dsp(id: Id, frequency: RealtimeAudioParameter) -> Dsp {
        let mut phase = 0.0f32;

        Dsp::new(
            id,
            Box::new(move |output: &mut dyn AudioBuffer| {
                let sample_rate = output.sample_rate() as f32;
                for frame in 0..output.num_frames() {
                    phase += 1.0;

                    let value = 0.5
                        * (std::f32::consts::TAU * frequency.float_value() * phase / sample_rate)
                            .sin();
                    for channel in 0..output.num_channels() {
                        let location = SampleLocation { channel, frame };
                        output.set_sample(&location, value);
                    }
                }
            }),
        )
    }
}
