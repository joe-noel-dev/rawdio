use std::time::Duration;

use crate::Level;

enum Phase {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

pub struct AdsrEnvelope {
    sample_rate: usize,
    phase: Phase,
    envelope: f64,
    attack_coefficient: f64,
    decay_coefficient: f64,
    sustain_level: Level,
    release_coefficient: f64,

    decay_time: Duration,
    release_time: Duration,
}

const ATTACK_TARGET: f64 = 1.1;
const DECAY_TARGET: f64 = -0.1;
const RELEASE_TARGET: f64 = -0.1;

impl AdsrEnvelope {
    pub fn new(
        sample_rate: usize,
        attack_time: Duration,
        decay_time: Duration,
        sustain_level: Level,
        release_time: Duration,
    ) -> Self {
        Self {
            sample_rate,
            phase: Phase::Idle,
            envelope: 0.0,
            attack_coefficient: calculate_attack_coefficient(attack_time, sample_rate),
            decay_coefficient: calculate_decay_coefficient(decay_time, sustain_level, sample_rate),
            sustain_level,
            release_coefficient: calculate_release_coefficient(
                release_time,
                sustain_level,
                sample_rate,
            ),
            decay_time,
            release_time,
        }
    }

    pub fn open(&mut self) {
        self.phase = Phase::Attack;
    }

    pub fn close(&mut self) {
        self.phase = Phase::Release;
    }

    pub fn set_attack_time(&mut self, attack_time: Duration) {
        self.attack_coefficient = calculate_attack_coefficient(attack_time, self.sample_rate);
    }

    pub fn set_decay_time(&mut self, decay_time: Duration) {
        self.decay_coefficient =
            calculate_decay_coefficient(decay_time, self.sustain_level, self.sample_rate);
    }

    pub fn set_sustain_level(&mut self, sustain_level: Level) {
        self.decay_coefficient =
            calculate_decay_coefficient(self.decay_time, sustain_level, self.sample_rate);
        self.release_coefficient =
            calculate_release_coefficient(self.release_time, sustain_level, self.sample_rate);
    }

    pub fn set_release_time(&mut self, release_time: Duration) {
        self.release_coefficient =
            calculate_release_coefficient(release_time, self.sustain_level, self.sample_rate);
    }

    pub fn process(&mut self) -> f64 {
        self.envelope = match self.phase {
            Phase::Idle => 0.0,
            Phase::Attack => {
                let output =
                    process_exponential(self.envelope, ATTACK_TARGET, self.attack_coefficient);
                let output = output.min(1.0);

                if output >= 1.0 {
                    self.phase = Phase::Decay;
                }

                output
            }
            Phase::Decay => {
                let output =
                    process_exponential(self.envelope, DECAY_TARGET, self.decay_coefficient);
                let output = output.max(self.sustain_level.as_gain());

                if output <= self.sustain_level.as_gain() {
                    self.phase = Phase::Sustain;
                }

                output
            }
            Phase::Sustain => self.sustain_level.as_gain(),
            Phase::Release => {
                let output =
                    process_exponential(self.envelope, RELEASE_TARGET, self.release_coefficient);
                let output = output.max(0.0);

                if output <= 0.0 {
                    self.phase = Phase::Idle;
                }

                output
            }
        };

        self.envelope
    }
}

fn process_exponential(previous_value: f64, target_value: f64, coefficient: f64) -> f64 {
    target_value + coefficient * (previous_value - target_value)
}

fn calculate_coefficient(log_term: f64, time: Duration, sample_rate: usize) -> f64 {
    if time.as_secs_f64() <= 0.0 {
        return 0.0;
    }

    (log_term.ln() / (time.as_secs_f64() * sample_rate as f64)).exp()
}

fn calculate_attack_coefficient(attack_time: Duration, sample_rate: usize) -> f64 {
    calculate_coefficient(
        (ATTACK_TARGET - 1.0) / ATTACK_TARGET,
        attack_time,
        sample_rate,
    )
}

fn calculate_decay_coefficient(
    decay_time: Duration,
    sustain_level: Level,
    sample_rate: usize,
) -> f64 {
    calculate_coefficient(
        (sustain_level.as_gain() - DECAY_TARGET) / (1.0 - DECAY_TARGET),
        decay_time,
        sample_rate,
    )
}

fn calculate_release_coefficient(
    release_time: Duration,
    sustain_level: Level,
    sample_rate: usize,
) -> f64 {
    calculate_coefficient(
        (-RELEASE_TARGET) / (sustain_level.as_gain() - RELEASE_TARGET),
        release_time,
        sample_rate,
    )
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::*;

    const SAMPLE_RATE: usize = 48_000;
    const ATTACK_TIME: Duration = Duration::from_millis(100);
    const DECAY_TIME: Duration = Duration::from_millis(200);
    const RELEASE_TIME: Duration = Duration::from_millis(500);
    const SUSTAIN_LEVEL: f64 = 0.5;

    struct Fixture {
        envelope: AdsrEnvelope,
    }

    impl Fixture {
        fn new() -> Self {
            Self {
                envelope: AdsrEnvelope::new(
                    SAMPLE_RATE,
                    ATTACK_TIME,
                    DECAY_TIME,
                    Level::from_gain(SUSTAIN_LEVEL),
                    RELEASE_TIME,
                ),
            }
        }

        fn note_on(&mut self) {
            self.envelope.open()
        }

        fn note_off(&mut self) {
            self.envelope.close()
        }

        fn process(&mut self, duration: Duration) -> Vec<f64> {
            let sample_count = (duration.as_secs_f64() * SAMPLE_RATE as f64).ceil() as usize;
            (0..sample_count).map(|_| self.envelope.process()).collect()
        }
    }

    #[test]
    fn attack_phase() {
        let mut fixture = Fixture::new();
        fixture.note_on();
        let values = fixture.process(ATTACK_TIME);

        assert!(values.iter().tuple_windows().all(|(a, b)| a <= b));
        assert!(values.iter().all(|value| 0.0 <= *value && *value <= 1.0));

        let next_values = fixture.process(Duration::from_secs(1));
        assert_eq!(*next_values.first().unwrap(), 1.0);
    }

    #[test]
    fn decay_phase() {
        let mut fixture = Fixture::new();
        fixture.note_on();

        fixture.process(ATTACK_TIME);
        let values = fixture.process(DECAY_TIME);

        assert!(values.iter().tuple_windows().all(|(a, b)| a >= b));
        assert!(values
            .iter()
            .all(|value| SUSTAIN_LEVEL <= *value && *value <= 1.0));

        let next_values = fixture.process(Duration::from_secs(1));
        assert_eq!(*next_values.first().unwrap(), SUSTAIN_LEVEL);
    }

    #[test]
    fn sustain_phase() {
        let mut fixture = Fixture::new();
        fixture.note_on();

        fixture.process(ATTACK_TIME);
        fixture.process(DECAY_TIME);
        let values = fixture.process(Duration::from_secs(1));

        assert!(values
            .iter()
            .all(|value| (value - SUSTAIN_LEVEL).abs() < 1e-3));
    }

    #[test]
    fn release_phase() {
        let mut fixture = Fixture::new();
        fixture.note_on();

        fixture.process(ATTACK_TIME);
        fixture.process(DECAY_TIME);
        fixture.process(Duration::from_secs(1));
        fixture.note_off();

        let values = fixture.process(RELEASE_TIME);

        assert!(values.iter().tuple_windows().all(|(a, b)| a >= b));
        assert!(values
            .iter()
            .all(|value| 0.0 <= *value && *value <= SUSTAIN_LEVEL));

        let next_values = fixture.process(Duration::from_secs(1));
        assert_eq!(*next_values.first().unwrap(), 0.0);
    }
}
