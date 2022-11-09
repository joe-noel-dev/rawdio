use crate::{
    commands::{
        command::{Command, ParameterChangeRequest},
        id::Id,
    },
    graph::node::CommandQueue,
    timestamp::Timestamp,
};
use atomic_float::AtomicF64;

use super::{realtime_parameter::RealtimeAudioParameter, ParameterChange};
use super::{ParameterValue, ValueChangeMethod};

pub struct AudioParameter {
    dsp_id: Id,
    parameter_id: Id,
    value: ParameterValue,
    minimum_value: f64,
    maximum_value: f64,
    command_queue: CommandQueue,
}

impl AudioParameter {
    pub fn new(
        dsp_id: Id,
        initial_value: f64,
        minimum_value: f64,
        maximum_value: f64,
        command_queue: CommandQueue,
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

            current_time = current_time.incremented_by_samples(1, 44_100);
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

            current_time = current_time.incremented_by_samples(1, 44_100);
        }
    }

    #[test]
    fn multiple_ramps() {
        let value = ParameterValue::new(AtomicF64::new(0.0));
        let parameter_id = Id::generate();
        let mut realtime_parameter = RealtimeAudioParameter::new(parameter_id, value);

        realtime_parameter.add_parameter_change(ParameterChange {
            value: 0.0,
            end_time: Timestamp::zero(),
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

            current_time = current_time.incremented_by_samples(1, 44_100);

            realtime_parameter.set_current_time(current_time);
        }
    }
}
