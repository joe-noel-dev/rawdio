use crate::{
    commands::{CancelChangeRequest, Command, Id, ParameterChangeRequest},
    CommandQueue, Context, Timestamp,
};
use atomic_float::AtomicF64;

use super::{parameter_change::ValueChangeMethod, parameter_value::ParameterValue, ParameterRange};
use super::{realtime_parameter::RealtimeAudioParameter, ParameterChange};

/// An parameter that generates a value for every audio sample
pub struct AudioParameter {
    dsp_id: Id,
    parameter_id: Id,
    value: ParameterValue,
    range: ParameterRange,
    command_queue: Box<dyn CommandQueue>,
}

impl AudioParameter {
    /// Create a new audio parameter
    pub fn new(
        dsp_id: Id,
        range: ParameterRange,
        context: &dyn Context,
    ) -> (Self, RealtimeAudioParameter) {
        let parameter_id = Id::generate();
        let value = ParameterValue::new(AtomicF64::new(range.default()));
        let realtime_audio_param =
            RealtimeAudioParameter::new(parameter_id, value.clone(), context.maximum_frame_count());

        (
            Self {
                dsp_id,
                parameter_id,
                value,
                range,
                command_queue: context.get_command_queue(),
            },
            realtime_audio_param,
        )
    }

    /// Get the ID of this parameters
    pub fn get_id(&self) -> Id {
        self.parameter_id
    }

    /// Get the last known value of this parameter
    pub fn get_value(&self) -> ParameterValue {
        self.value.clone()
    }

    /// Set the value of this parameter now
    pub fn set_value_now(&mut self, value: f64) {
        self.set_value_at_time(value, Timestamp::zero());
    }

    /// Set the value of this parameter at a particular time
    ///
    /// If `at_time` is in the past, it will be handled immediately
    pub fn set_value_at_time(&mut self, mut value: f64, at_time: Timestamp) {
        value = self.range.clamp(value);
        self.command_queue
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

    /// Linearly to a value over a time window
    pub fn linear_ramp_to_value(&mut self, value: f64, start_time: Timestamp, end_time: Timestamp) {
        let value = self.range.clamp(value);

        self.command_queue
            .send(Command::ParameterValueChange(ParameterChangeRequest {
                dsp_id: self.dsp_id,
                parameter_id: self.parameter_id,
                change: ParameterChange {
                    value,
                    end_time,
                    method: ValueChangeMethod::Linear(start_time),
                },
            }));
    }

    /// Exponentially ramp to a value over a time window
    ///
    /// It is not possible to ramp to or from 0
    pub fn exponential_ramp_to_value(
        &mut self,
        value: f64,
        start_time: Timestamp,
        end_time: Timestamp,
    ) {
        let value = self.range.clamp(value);

        self.command_queue
            .send(Command::ParameterValueChange(ParameterChangeRequest {
                dsp_id: self.dsp_id,
                parameter_id: self.parameter_id,
                change: ParameterChange {
                    value,
                    end_time,
                    method: ValueChangeMethod::Exponential(start_time),
                },
            }));
    }

    /// Cancel all scheduled changes
    pub fn cancel_scheduled_changes(&mut self) {
        self.command_queue
            .send(Command::CancelParameterChanges(CancelChangeRequest {
                dsp_id: self.dsp_id,
                parameter_id: self.parameter_id,
                end_time: None,
            }));
    }

    /// Cancel all scheduled changes that complete after `end_time`
    pub fn cancel_scheduled_changes_ending_after(&mut self, end_time: Timestamp) {
        self.command_queue
            .send(Command::CancelParameterChanges(CancelChangeRequest {
                dsp_id: self.dsp_id,
                parameter_id: self.parameter_id,
                end_time: Some(end_time),
            }));
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;

    struct Fixture {
        realtime_parameter: RealtimeAudioParameter,
    }

    impl Fixture {
        fn new(initial_value: f64) -> Self {
            let id = Id::generate();
            let value = ParameterValue::new(AtomicF64::new(initial_value));
            let maximum_frame_count = 512;
            let realtime_parameter = RealtimeAudioParameter::new(id, value, maximum_frame_count);

            Self { realtime_parameter }
        }

        fn get_parameter_values(
            &mut self,
            start_time: Timestamp,
            end_time: Timestamp,
            sample_rate: usize,
        ) -> Vec<f32> {
            let mut values = Vec::new();

            let start_sample = start_time.as_samples(sample_rate).ceil() as usize;
            let end_sample = end_time.as_samples(sample_rate).ceil() as usize;
            let frame_size = 512;

            for frame in (start_sample..end_sample).step_by(frame_size) {
                let frame_end_sample = (frame + frame_size).min(end_sample);
                let current_time = start_time.incremented_by_samples(frame, sample_rate);
                let frame_count = frame_end_sample - frame;

                self.realtime_parameter
                    .process(&current_time, frame_count, sample_rate);

                values.extend_from_slice(self.realtime_parameter.get_values(frame_count));
            }

            values
        }
    }

    #[test]
    fn immediate_change() {
        let mut fixture = Fixture::new(6.0);

        fixture
            .realtime_parameter
            .add_parameter_change(ParameterChange {
                value: 7.0,
                end_time: Timestamp::from_seconds(1.0),
                method: ValueChangeMethod::Immediate,
            });

        let sample_rate = 44_100;
        let values = fixture.get_parameter_values(
            Timestamp::zero(),
            Timestamp::from_seconds(2.0),
            sample_rate,
        );

        for (frame_index, value) in values.iter().enumerate() {
            let current_time = Timestamp::from_samples(frame_index as f64, sample_rate);

            let expected_value = if current_time.as_seconds() < 1.0 {
                6.0
            } else {
                7.0
            };

            assert_relative_eq!(*value, expected_value, epsilon = 1e-6);
        }
    }

    #[test]
    fn linear_ramp() {
        let mut fixture = Fixture::new(5.0);

        let start_time = Timestamp::zero();
        let end_time = Timestamp::from_seconds(5.0);

        fixture
            .realtime_parameter
            .add_parameter_change(ParameterChange {
                value: 10.0,
                end_time,
                method: ValueChangeMethod::Linear(start_time),
            });

        let sample_rate = 44_100;
        let values = fixture.get_parameter_values(
            Timestamp::zero(),
            Timestamp::from_seconds(20.0),
            sample_rate,
        );

        for (frame_index, value) in values.iter().enumerate() {
            let current_time = Timestamp::zero().incremented_by_samples(frame_index, sample_rate);

            let expected_value = if current_time <= end_time {
                current_time.as_seconds() + 5.0
            } else {
                10.0
            };

            assert_relative_eq!(*value, expected_value as f32, epsilon = 1e-3);
        }
    }

    #[test]
    fn multiple_ramps() {
        let mut fixture = Fixture::new(0.0);

        let changes = [
            (0.0, Timestamp::zero(), ValueChangeMethod::Immediate),
            (
                1.0,
                Timestamp::from_seconds(1.0),
                ValueChangeMethod::Linear(Timestamp::zero()),
            ),
            (
                0.0,
                Timestamp::from_seconds(2.0),
                ValueChangeMethod::Linear(Timestamp::from_seconds(1.0)),
            ),
            (
                1.0,
                Timestamp::from_seconds(3.0),
                ValueChangeMethod::Linear(Timestamp::from_seconds(2.0)),
            ),
            (
                0.0,
                Timestamp::from_seconds(4.0),
                ValueChangeMethod::Linear(Timestamp::from_seconds(3.0)),
            ),
            (
                1.0,
                Timestamp::from_seconds(5.0),
                ValueChangeMethod::Linear(Timestamp::from_seconds(4.0)),
            ),
        ];

        for (value, end_time, method) in changes {
            fixture
                .realtime_parameter
                .add_parameter_change(ParameterChange {
                    value,
                    end_time,
                    method,
                });
        }

        let sample_rate = 44_100;

        let values = fixture.get_parameter_values(
            Timestamp::zero(),
            Timestamp::from_seconds(6.0),
            sample_rate,
        );

        for (frame_index, value) in values.iter().enumerate() {
            let current_seconds =
                Timestamp::from_samples(frame_index as f64, sample_rate).as_seconds();

            let expected = if (0.0..1.0).contains(&current_seconds)
                || (2.0..3.0).contains(&current_seconds)
                || (4.0..5.0).contains(&current_seconds)
            {
                current_seconds % 1.0
            } else if current_seconds >= 5.0 {
                1.0
            } else {
                1.0 - current_seconds % 1.0
            };

            assert_relative_eq!(*value, expected as f32, epsilon = 1e-3);
        }
    }

    #[test]
    fn cancel_scheduled_changes() {
        let mut fixture = Fixture::new(2.0);

        fixture
            .realtime_parameter
            .add_parameter_change(ParameterChange {
                value: 5.0,
                end_time: Timestamp::from_seconds(1.0),
                method: ValueChangeMethod::Immediate,
            });

        fixture.realtime_parameter.cancel_scheduled_changes();

        let start_time = Timestamp::zero();
        let end_time = Timestamp::from_seconds(2.0);
        let sample_rate = 48_000;
        let values = fixture.get_parameter_values(start_time, end_time, sample_rate);

        for value in values {
            assert_relative_eq!(value, 2.0);
        }
    }
}
