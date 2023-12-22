use crate::Timestamp;

use super::{
    parameter_change::ValueChangeMethod, parameter_value::ParameterValue, ParameterChange,
    ParameterId,
};

use std::sync::atomic::Ordering;

const MAXIMUM_PENDING_PARAMETER_CHANGES: usize = 16;

#[repr(align(64))]
struct ParameterBuffer {
    values: Vec<f32>,
}

impl ParameterBuffer {
    fn with_capacity(capacity: usize) -> Self {
        Self {
            values: Vec::with_capacity(capacity),
        }
    }

    fn reset(&mut self) {
        self.values.clear()
    }

    fn add_value(&mut self, value: f32) {
        self.values.push(value)
    }

    fn get_values(&self, frame_count: usize) -> &[f32] {
        &self.values[..frame_count]
    }

    fn fill(&mut self, value: f32, frame_count: usize) {
        self.values.resize(frame_count, value);
    }
}

pub struct RealtimeAudioParameter {
    id: ParameterId,
    value: ParameterValue,
    parameter_changes: Vec<ParameterChange>,
    parameter_buffer: ParameterBuffer,
    increment: f64,
    coefficient: f64,
    current_change: ParameterChange,
}

impl RealtimeAudioParameter {
    pub fn new(id: ParameterId, value: ParameterValue, maximum_frame_count: usize) -> Self {
        let initial_value = value.load(Ordering::Acquire);

        Self {
            id,
            value,
            parameter_changes: Vec::with_capacity(MAXIMUM_PENDING_PARAMETER_CHANGES),
            parameter_buffer: ParameterBuffer::with_capacity(maximum_frame_count),
            increment: 0.0,
            coefficient: 1.0,
            current_change: ParameterChange {
                value: initial_value,
                end_time: Timestamp::zero(),
                method: ValueChangeMethod::Immediate,
            },
        }
    }

    pub fn get_id(&self) -> ParameterId {
        self.id
    }

    pub fn get_value(&self) -> f64 {
        self.value.load(Ordering::Acquire)
    }

    fn is_static(&self) -> bool {
        if (self.coefficient - 1.0).abs() > 1e-6 {
            return false;
        }

        if self.increment.abs() > 1e-6 {
            return false;
        }

        if !self.parameter_changes.is_empty() {
            return false;
        }

        true
    }

    pub fn process(&mut self, time: &Timestamp, frame_count: usize, sample_rate: usize) {
        self.parameter_buffer.reset();

        let mut value = self.get_value();

        if self.is_static() {
            self.parameter_buffer.fill(value as f32, frame_count);
        } else {
            for frame in 0..frame_count {
                let frame_time = time.incremented_by_samples(frame, sample_rate);
                value = self.value_at_time(&frame_time, sample_rate, value);
                self.parameter_buffer.add_value(value as f32);
            }
        }

        self.set_value(value);
    }

    fn value_at_time(&mut self, time: &Timestamp, sample_rate: usize, value: f64) -> f64 {
        if let Some(next_event) = self.parameter_changes.first() {
            match next_event.method {
                ValueChangeMethod::Immediate => {
                    if next_event.end_time <= *time {
                        self.increment = 0.0;
                        self.coefficient = 1.0;
                        self.current_change = self.parameter_changes.remove(0);
                    }
                }
                ValueChangeMethod::Linear(start_time) => {
                    if *time >= start_time {
                        let duration = next_event.end_time.as_samples(sample_rate)
                            - time.as_samples(sample_rate);
                        let delta = next_event.value - value;

                        self.increment = delta / duration;
                        self.coefficient = 1.0;
                        self.current_change = self.parameter_changes.remove(0);
                    }
                }
                ValueChangeMethod::Exponential(start_time) => {
                    if *time >= start_time {
                        debug_assert_ne!(next_event.value, 0.0);
                        debug_assert_ne!(value, 0.0);

                        let ratio = next_event.value / value;

                        let duration = next_event.end_time.as_samples(sample_rate)
                            - time.as_samples(sample_rate);

                        self.increment = 0.0;
                        self.coefficient = (ratio.ln() / duration).exp();

                        self.current_change = self.parameter_changes.remove(0);
                    }
                }
            };
        }

        if self.current_change.end_time <= *time {
            return self.current_change.value;
        }

        (value * self.coefficient) + self.increment
    }

    pub fn get_values(&self, frame_count: usize) -> &[f32] {
        self.parameter_buffer.get_values(frame_count)
    }

    pub fn set_value(&mut self, value: f64) {
        self.value.store(value, Ordering::Release)
    }

    pub fn add_parameter_change(&mut self, parameter_change: ParameterChange) {
        self.parameter_changes.push(parameter_change);

        self.parameter_changes
            .sort_by(|a, b| a.end_time.partial_cmp(&b.end_time).unwrap());
    }

    pub fn cancel_scheduled_changes_ending_after(&mut self, time: &Timestamp) {
        self.parameter_changes
            .retain(|change| change.end_time >= *time);
    }

    pub fn cancel_scheduled_changes(&mut self) {
        self.parameter_changes.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use atomic_float::AtomicF64;

    fn process_parameter_values(
        parameter: &mut RealtimeAudioParameter,
        from_time: Timestamp,
        to_time: Timestamp,
        sample_rate: usize,
    ) -> Vec<f32> {
        let mut values = Vec::new();

        let start_sample = from_time.as_samples(sample_rate).ceil() as usize;
        let end_sample = to_time.as_samples(sample_rate).ceil() as usize;
        let maximum_frame_count = 512;

        for frame in (start_sample..end_sample).step_by(maximum_frame_count) {
            let frame_end_sample = (frame + maximum_frame_count).min(end_sample);
            let current_time = from_time.incremented_by_samples(frame, sample_rate);
            let frame_count = frame_end_sample - frame;

            parameter.process(&current_time, frame_count, sample_rate);

            values.extend_from_slice(parameter.get_values(frame_count));
        }

        values
    }

    #[test]
    fn immediate_parameter_changes() {
        let value = ParameterValue::new(AtomicF64::new(0.0));
        let maximum_frame_count = 512;

        let mut param = RealtimeAudioParameter::new("param", value, maximum_frame_count);

        param.add_parameter_change(ParameterChange {
            value: 1.0,
            end_time: Timestamp::from_seconds(1.0),
            method: ValueChangeMethod::Immediate,
        });

        param.add_parameter_change(ParameterChange {
            value: 2.0,
            end_time: Timestamp::from_seconds(2.0),
            method: ValueChangeMethod::Immediate,
        });

        param.add_parameter_change(ParameterChange {
            value: 3.0,
            end_time: Timestamp::from_seconds(3.0),
            method: ValueChangeMethod::Immediate,
        });

        let sample_rate = 48_000;
        let max_time = 3.5;
        let values = process_parameter_values(
            &mut param,
            Timestamp::zero(),
            Timestamp::from_seconds(max_time),
            sample_rate,
        );

        let get_value_at_time = |time: f64| {
            let offset = Timestamp::from_seconds(time).as_samples(sample_rate).ceil() as usize;
            assert!(offset < values.len());
            values[offset]
        };

        assert_relative_eq!(get_value_at_time(0.9), 0.0);
        assert_relative_eq!(get_value_at_time(1.0), 1.0);
        assert_relative_eq!(get_value_at_time(1.9), 1.0);
        assert_relative_eq!(get_value_at_time(2.0), 2.0);
        assert_relative_eq!(get_value_at_time(2.9), 2.0);
        assert_relative_eq!(get_value_at_time(3.0), 3.0);
    }

    #[test]
    fn ramped_parameter_changes() {
        let value = ParameterValue::new(AtomicF64::new(0.0));
        let maximum_frame_count = 512;

        let mut param = RealtimeAudioParameter::new("param", value, maximum_frame_count);

        [
            (1.0, Timestamp::zero(), Timestamp::from_seconds(1.0)),
            (
                2.0,
                Timestamp::from_seconds(1.0),
                Timestamp::from_seconds(2.0),
            ),
            (
                3.0,
                Timestamp::from_seconds(2.0),
                Timestamp::from_seconds(3.0),
            ),
        ]
        .iter()
        .for_each(|(value, start_time, end_time)| {
            param.add_parameter_change(ParameterChange {
                value: *value,
                end_time: *end_time,
                method: ValueChangeMethod::Linear(*start_time),
            });
        });

        let sample_rate = 48_000;
        let values = process_parameter_values(
            &mut param,
            Timestamp::zero(),
            Timestamp::from_seconds(3.5),
            sample_rate,
        );

        let get_value_at_time = |time: f64| {
            let offset = Timestamp::from_seconds(time).as_samples(sample_rate).ceil() as usize;
            values[offset]
        };

        assert_relative_eq!(get_value_at_time(0.5), 0.5, epsilon = 1e-3);
        assert_relative_eq!(get_value_at_time(1.0), 1.0, epsilon = 1e-3);
        assert_relative_eq!(get_value_at_time(1.5), 1.5, epsilon = 1e-3);
        assert_relative_eq!(get_value_at_time(2.0), 2.0, epsilon = 1e-3);
        assert_relative_eq!(get_value_at_time(2.5), 2.5, epsilon = 1e-3);
        assert_relative_eq!(get_value_at_time(3.0), 3.0, epsilon = 1e-3);
    }

    #[test]
    fn exponential_ramps() {
        let initial_value = 2.0;
        let value = ParameterValue::new(AtomicF64::new(initial_value));
        let maximum_frame_count = 512;
        let mut param = RealtimeAudioParameter::new("param", value, maximum_frame_count);

        let ramp_duration = Timestamp::from_seconds(1.0);

        param.add_parameter_change(ParameterChange {
            value: 2.0 * initial_value,
            end_time: ramp_duration,
            method: ValueChangeMethod::Exponential(Timestamp::zero()),
        });

        let sample_rate = 96_000;

        let values = process_parameter_values(
            &mut param,
            Timestamp::zero(),
            ramp_duration.incremented_by_seconds(0.1),
            sample_rate,
        );

        let get_value_at_time = |time: f64| {
            let offset = Timestamp::from_seconds(time).as_samples(sample_rate).ceil() as usize;
            values[offset]
        };

        assert_relative_eq!(get_value_at_time(0.0), 2.0, epsilon = 1e-3);
        assert_relative_eq!(get_value_at_time(0.5), 2.0 * 1.414, epsilon = 1e-3);
        assert_relative_eq!(get_value_at_time(1.0), 4.0, epsilon = 1e-3);
    }

    #[test]
    fn zero_time_change_is_immediate() {
        let value = ParameterValue::new(AtomicF64::new(0.0));
        let maximum_frame_count = 512;

        let mut param = RealtimeAudioParameter::new("param", value, maximum_frame_count);

        param.add_parameter_change(ParameterChange {
            value: 1.0,
            end_time: Timestamp::zero(),
            method: ValueChangeMethod::Immediate,
        });

        let sample_rate = 48_000;

        let values = process_parameter_values(
            &mut param,
            Timestamp::from_seconds(2.0),
            Timestamp::from_seconds(4.5),
            sample_rate,
        );

        let get_value_at_time = |time: f64| {
            let offset = Timestamp::from_seconds(time).as_samples(sample_rate).ceil() as usize;
            assert!(offset < values.len());
            values[offset]
        };

        assert_relative_eq!(get_value_at_time(0.0), 1.0);
        assert_relative_eq!(get_value_at_time(1.0), 1.0);
        assert_relative_eq!(get_value_at_time(2.0), 1.0);
    }
}
