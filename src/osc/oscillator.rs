use crate::{commands::id::Id, graph::node::Node, parameter::Parameter};

#[derive(Clone)]
pub enum OscillatorType {
    Sine,
}

#[derive(Clone)]
pub struct Oscillator {
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
    pub fn new(id: Id) -> Self {
        Self {
            id,
            oscillator_type: OscillatorType::Sine,
            frequency: Parameter::Double(440.0),
        }
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
