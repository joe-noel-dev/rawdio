use crate::{node::Node, parameter::Parameter};

#[derive(Clone)]
pub enum OscillatorType {
    Sine,
}

#[derive(Clone)]
pub struct Oscillator {
    oscillator_type: OscillatorType,
    frequency: Parameter,
}

impl Default for Oscillator {
    fn default() -> Self {
        Self {
            oscillator_type: OscillatorType::Sine,
            frequency: Parameter::Double(440.0),
        }
    }
}

impl Node for Oscillator {}

impl Oscillator {
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
