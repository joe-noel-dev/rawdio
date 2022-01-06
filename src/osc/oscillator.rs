use lockfree::channel::mpsc::Sender;

use crate::{
    commands::{command::Command, id::Id},
    graph::node::Node,
    parameter::Parameter,
};

use super::realtime_oscillator::RealtimeOscillator;

#[derive(Clone)]
pub enum OscillatorType {
    Sine,
}

#[derive(Clone)]
pub struct Oscillator {
    command_queue: Sender<Command>,
    id: Id,
    oscillator_type: OscillatorType,
    frequency: Parameter,
}

impl Node for Oscillator {
    fn get_id(&self) -> Id {
        self.id
    }
}

impl Oscillator {
    pub fn new(command_queue: Sender<Command>, sample_rate: f32) -> Self {
        let id = Id::generate();

        let realtime_osc = RealtimeOscillator::new(id, sample_rate);

        let _ = command_queue.send(Command::AddOscillator(realtime_osc));

        Self {
            command_queue,
            id,
            oscillator_type: OscillatorType::Sine,
            frequency: Parameter::Double(440.0),
        }
    }

    pub fn remove(&mut self) {
        let _ = self.command_queue.send(Command::RemoveOscillator(self.id));
    }

    pub fn with_type(&self, osillator_type: OscillatorType) -> Self {
        let mut other = self.clone();
        other.oscillator_type = osillator_type;
        other
    }

    pub fn with_frequency(&self, frequency: f64) -> Self {
        let mut other = self.clone();
        other.frequency = Parameter::Double(frequency);
        other
    }
}
