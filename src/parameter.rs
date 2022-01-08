use std::sync::{atomic::Ordering, Arc};

use lockfree::channel::mpsc::Sender;

use crate::{
    commands::{command::Command, id::Id},
    timestamp::Timestamp,
};
use atomic_float::AtomicF64;

pub type ParameterValue = Arc<AtomicF64>;

pub struct AudioParameter {
    id: Id,
    value: ParameterValue,
    command_queue: Sender<Command>,
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct ParameterChange {
    value: f64,
    end_time: Timestamp,
}

pub struct RealtimeAudioParameter {
    id: Id,
    value: ParameterValue,
    parameter_changes: Vec<ParameterChange>,
    current_time: Timestamp,
}

impl AudioParameter {
    pub fn new(initial_value: f64, command_queue: Sender<Command>) -> Self {
        let id = Id::generate();
        let param_value = ParameterValue::new(AtomicF64::new(initial_value));
        let realtime_audio_param = RealtimeAudioParameter::new(id, param_value.clone());

        let _ = command_queue.send(Command::AddParameter(Box::new(realtime_audio_param)));

        Self {
            id,
            value: param_value,
            command_queue,
        }
    }

    pub fn get_value(&self) -> ParameterValue {
        self.value.clone()
    }

    pub fn set_value_immediate(&mut self, value: f64) {
        let _ = self
            .command_queue
            .send(Command::SetValueImmediate((self.id, value)));
    }

    pub fn linear_ramp_to_value(&mut self, value: f64, time: Timestamp) {
        let _ = self
            .command_queue
            .send(Command::LinearRampToValue((self.id, value, time)));
    }
}

impl Drop for AudioParameter {
    fn drop(&mut self) {
        let _ = self.command_queue.send(Command::RemoveParameter(self.id));
    }
}

impl RealtimeAudioParameter {
    pub fn new(id: Id, value: ParameterValue) -> Self {
        let mut parameter_changes = Vec::new();
        parameter_changes.reserve(16);

        Self {
            id,
            value,
            parameter_changes,
            current_time: Timestamp::default(),
        }
    }

    pub fn get_id(&self) -> Id {
        self.id
    }

    pub fn get_value(&self) -> f64 {
        self.value.load(Ordering::Acquire)
    }

    pub fn jump_to_time(&mut self, time: &Timestamp) {
        assert!(time >= &self.current_time);

        let mut previous = self.get_value();
        let mut start_time = self.current_time;
        let mut next_change: Option<ParameterChange> = None;

        for change in self.parameter_changes.iter() {
            if change.end_time <= *time {
                previous = change.value;
                start_time = change.end_time;
            } else {
                next_change = Some(*change);
                break;
            }
        }

        if let Some(change) = next_change {
            let a = (change.value - previous)
                / (change.end_time.get_seconds() - start_time.get_seconds());
            let b = previous - a * start_time.get_seconds();
            let new_value = a * time.get_seconds() + b;
            self.set_value(new_value);
        } else {
            self.set_value(previous);
        }

        self.current_time = *time;

        self.remove_expired_parameter_changes();
    }

    fn remove_expired_parameter_changes(&mut self) {
        self.parameter_changes
            .retain(|change| change.end_time > self.current_time);
    }

    pub fn set_value(&mut self, value: f64) {
        self.value.store(value, Ordering::Release)
    }

    pub fn add_parameter_change(&mut self, value: f64, end_time: Timestamp) {
        self.parameter_changes
            .push(ParameterChange { value, end_time });

        self.parameter_changes
            .sort_by(|a, b| a.end_time.partial_cmp(&b.end_time).unwrap());
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;
    #[test]
    fn lineear_ramps() {
        let value = ParameterValue::new(AtomicF64::new(5.0));
        let id = Id::generate();
        let mut realtime_parameter = RealtimeAudioParameter::new(id, value);
        let end_time = Timestamp::from_seconds(5.0);
        realtime_parameter.add_parameter_change(10.0, end_time);

        let mut current_time = Timestamp::default();
        while current_time <= Timestamp::from_seconds(20.0) {
            let expected_value = if current_time <= end_time {
                current_time.get_seconds() + 5.0
            } else {
                10.0
            };

            assert_relative_eq!(
                realtime_parameter.get_value(),
                expected_value,
                epsilon = 1e-6
            );

            current_time = current_time.incremented_by_samples(1, 44100);
            realtime_parameter.jump_to_time(&current_time);
        }
    }
}
