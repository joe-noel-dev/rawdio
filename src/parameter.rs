use std::sync::{atomic::Ordering, Arc};

use lockfree::channel::mpsc::Sender;

use crate::{
    commands::{
        command::{Command, ParameterChangeRequest},
        id::Id,
    },
    timestamp::Timestamp,
};
use atomic_float::AtomicF64;

pub type ParameterValue = Arc<AtomicF64>;

pub struct AudioParameter {
    dsp_id: Id,
    parameter_id: Id,
    value: ParameterValue,
    minimum_value: f64,
    maximum_value: f64,
    command_queue: Sender<Command>,
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub enum ValueChangeMethod {
    Immediate,
    Linear,
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct ParameterChange {
    value: f64,
    end_time: Timestamp,
    method: ValueChangeMethod,
}

pub struct RealtimeAudioParameter {
    parameter_id: Id,
    value: ParameterValue,
    parameter_changes: Vec<ParameterChange>,
    last_value: f64,
    last_change: Timestamp,
}

impl AudioParameter {
    pub fn new(
        dsp_id: Id,
        initial_value: f64,
        minimum_value: f64,
        maximum_value: f64,
        command_queue: Sender<Command>,
    ) -> (Self, RealtimeAudioParameter) {
        assert!((minimum_value..maximum_value).contains(&initial_value));
        assert!(minimum_value < maximum_value);

        let parameter_id = Id::generate();
        let param_value = ParameterValue::new(AtomicF64::new(initial_value));
        let realtime_audio_param = RealtimeAudioParameter::new(parameter_id, param_value.clone());

        (
            Self {
                dsp_id,
                parameter_id,
                value: param_value,
                minimum_value,
                maximum_value,
                command_queue,
            },
            realtime_audio_param,
        )
    }

    pub fn get_id(&self) -> Id {
        self.parameter_id
    }

    pub fn get_value(&self) -> ParameterValue {
        self.value.clone()
    }

    pub fn set_value_at_time(&mut self, mut value: f64, at_time: Timestamp) {
        value = value.clamp(self.minimum_value, self.maximum_value);
        let _ = self
            .command_queue
            .send(Command::ParameterValueChange(ParameterChangeRequest {
                dsp_id: self.dsp_id,
                parameter_id: self.parameter_id,
                change: ParameterChange {
                    value,
                    end_time: at_time,
                    method: ValueChangeMethod::Immediate,
                },
            }));
    }

    pub fn linear_ramp_to_value(&mut self, mut value: f64, end_time: Timestamp) {
        value = value.clamp(self.minimum_value, self.maximum_value);
        let _ = self
            .command_queue
            .send(Command::ParameterValueChange(ParameterChangeRequest {
                dsp_id: self.dsp_id,
                parameter_id: self.parameter_id,
                change: ParameterChange {
                    value,
                    end_time,
                    method: ValueChangeMethod::Linear,
                },
            }));
    }
}

impl RealtimeAudioParameter {
    pub fn new(parameter_id: Id, value: ParameterValue) -> Self {
        let mut parameter_changes = Vec::new();
        parameter_changes.reserve(16);

        let initial_value = value.load(Ordering::Acquire);

        Self {
            parameter_id,
            value,
            parameter_changes,
            last_change: Timestamp::default(),
            last_value: initial_value,
        }
    }

    pub fn get_id(&self) -> Id {
        self.parameter_id
    }

    pub fn get_value(&self) -> f64 {
        self.value.load(Ordering::Acquire)
    }

    pub fn set_current_time(&mut self, time: Timestamp) {
        for param_change in self.parameter_changes.iter() {
            if param_change.end_time <= time {
                self.last_change = param_change.end_time;
                self.last_value = param_change.value;
            }
        }

        self.set_value(self.get_value_at_time(&time));

        self.parameter_changes
            .retain(|param_change| param_change.end_time > time);
    }

    pub fn get_value_at_time(&self, time: &Timestamp) -> f64 {
        let (previous_change, next_change) = self.get_next_parameter_change_after(time);

        if let Some(next_change) = next_change {
            return Self::get_next_value(previous_change, next_change, time);
        }

        previous_change.value
    }

    fn get_next_parameter_change_after(
        &self,
        time: &Timestamp,
    ) -> (ParameterChange, Option<ParameterChange>) {
        let mut previous_change = ParameterChange {
            value: self.last_value,
            end_time: self.last_change,
            method: ValueChangeMethod::Immediate,
        };

        let mut next_change: Option<ParameterChange> = None;

        for change in self.parameter_changes.iter() {
            if change.end_time <= *time {
                previous_change = *change;
            } else {
                next_change = Some(*change);
                break;
            }
        }

        (previous_change, next_change)
    }

    fn get_next_value(
        previous_change: ParameterChange,
        next_change: ParameterChange,
        time: &Timestamp,
    ) -> f64 {
        match next_change.method {
            ValueChangeMethod::Immediate => {
                if next_change.end_time <= *time {
                    next_change.value
                } else {
                    previous_change.value
                }
            }
            ValueChangeMethod::Linear => {
                let a = (next_change.value - previous_change.value)
                    / (next_change.end_time.get_seconds() - previous_change.end_time.get_seconds());
                let b = previous_change.value - a * previous_change.end_time.get_seconds();
                a * time.get_seconds() + b
            }
        }
    }

    pub fn set_value(&mut self, value: f64) {
        self.value.store(value, Ordering::Release)
    }

    pub fn add_parameter_change(&mut self, parameter_change: ParameterChange) {
        self.parameter_changes.push(parameter_change);

        self.parameter_changes
            .sort_by(|a, b| a.end_time.partial_cmp(&b.end_time).unwrap());
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;

    #[test]
    fn immediate_change() {
        let value = ParameterValue::new(AtomicF64::new(6.0));
        let parameter_id = Id::generate();
        let mut realtime_parameter = RealtimeAudioParameter::new(parameter_id, value);
        realtime_parameter.add_parameter_change(ParameterChange {
            value: 7.0,
            end_time: Timestamp::from_seconds(1.0),
            method: ValueChangeMethod::Immediate,
        });

        let mut current_time = Timestamp::default();
        while current_time <= Timestamp::from_seconds(2.0) {
            let expected_value = if current_time.get_seconds() < 1.0 {
                6.0
            } else {
                7.0
            };

            assert_relative_eq!(
                realtime_parameter.get_value_at_time(&current_time),
                expected_value,
                epsilon = 1e-6
            );

            current_time = current_time.incremented_by_samples(1, 44100.0);
        }
    }

    #[test]
    fn linear_ramp() {
        let value = ParameterValue::new(AtomicF64::new(5.0));
        let parameter_id = Id::generate();
        let mut realtime_parameter = RealtimeAudioParameter::new(parameter_id, value);
        let end_time = Timestamp::from_seconds(5.0);
        realtime_parameter.add_parameter_change(ParameterChange {
            value: 10.0,
            end_time,
            method: ValueChangeMethod::Linear,
        });

        let mut current_time = Timestamp::default();
        while current_time <= Timestamp::from_seconds(20.0) {
            let expected_value = if current_time <= end_time {
                current_time.get_seconds() + 5.0
            } else {
                10.0
            };

            assert_relative_eq!(
                realtime_parameter.get_value_at_time(&current_time),
                expected_value,
                epsilon = 1e-6
            );

            current_time = current_time.incremented_by_samples(1, 44100.0);
        }
    }

    #[test]
    fn multiple_ramps() {
        let value = ParameterValue::new(AtomicF64::new(0.0));
        let parameter_id = Id::generate();
        let mut realtime_parameter = RealtimeAudioParameter::new(parameter_id, value);

        realtime_parameter.add_parameter_change(ParameterChange {
            value: 0.0,
            end_time: Timestamp::from_seconds(0.0),
            method: ValueChangeMethod::Immediate,
        });

        realtime_parameter.add_parameter_change(ParameterChange {
            value: 1.0,
            end_time: Timestamp::from_seconds(1.0),
            method: ValueChangeMethod::Linear,
        });

        realtime_parameter.add_parameter_change(ParameterChange {
            value: 0.0,
            end_time: Timestamp::from_seconds(2.0),
            method: ValueChangeMethod::Linear,
        });

        realtime_parameter.add_parameter_change(ParameterChange {
            value: 1.0,
            end_time: Timestamp::from_seconds(3.0),
            method: ValueChangeMethod::Linear,
        });

        realtime_parameter.add_parameter_change(ParameterChange {
            value: 0.0,
            end_time: Timestamp::from_seconds(4.0),
            method: ValueChangeMethod::Linear,
        });

        realtime_parameter.add_parameter_change(ParameterChange {
            value: 1.0,
            end_time: Timestamp::from_seconds(5.0),
            method: ValueChangeMethod::Linear,
        });

        let mut current_time = Timestamp::default();
        while current_time <= Timestamp::from_seconds(5.0) {
            let current_seconds = current_time.get_seconds();
            let expected_value = if (0.0..=1.0).contains(&current_seconds)
                || (2.0..=3.0).contains(&current_seconds)
                || (4.0..=5.0).contains(&current_seconds)
            {
                current_seconds % 1.0
            } else {
                1.0 - current_seconds % 1.0
            };

            assert_relative_eq!(
                realtime_parameter.get_value_at_time(&current_time),
                expected_value,
                epsilon = 1e-6
            );

            current_time = current_time.incremented_by_samples(1, 44100.0);

            realtime_parameter.set_current_time(current_time);
        }
    }
}
