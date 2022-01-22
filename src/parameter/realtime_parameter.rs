use crate::{commands::id::Id, Timestamp};

use super::{ParameterChange, ParameterValue, ValueChangeMethod};

use std::sync::atomic::Ordering;

pub struct RealtimeAudioParameter {
    parameter_id: Id,
    value: ParameterValue,
    parameter_changes: Vec<ParameterChange>,
    last_value: f64,
    last_change: Timestamp,
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
