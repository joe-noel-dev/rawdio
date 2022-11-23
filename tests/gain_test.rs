use approx::assert_relative_eq;
use rust_audio_engine::{
    create_engine, AudioBuffer, AudioProcess, Context, Gain, Level, Oscillator, OwnedAudioBuffer,
    SampleLocation, Timestamp,
};

struct Fixture {
    channel_count: usize,
    sample_rate: usize,
    context: Box<dyn Context>,
    audio_process: Box<dyn AudioProcess>,
    gain: Gain,
    _oscillator: Oscillator,
}

impl Fixture {
    fn process_seconds(&mut self, seconds: f64) -> OwnedAudioBuffer {
        let frame_count = (seconds * self.sample_rate as f64).ceil();
        let mut output_buffer =
            OwnedAudioBuffer::new(frame_count as usize, self.channel_count, self.sample_rate);
        self.audio_process.process(&mut output_buffer);
        output_buffer
    }
}

impl Default for Fixture {
    fn default() -> Self {
        let sample_rate = 44100;
        let channel_count = 1;
        let oscillator_frequency = 1_000.0;

        let (mut context, process) = create_engine(sample_rate);

        let oscillator = Oscillator::new(
            context.get_command_queue(),
            oscillator_frequency,
            channel_count,
        );

        let gain = Gain::new(context.get_command_queue(), channel_count);

        oscillator.node.connect_to(&gain.node);
        gain.node.connect_to_output();

        context.start();

        Self {
            channel_count,
            sample_rate,
            context,
            audio_process: process,
            gain,
            _oscillator: oscillator,
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        self.context.stop();
    }
}

fn create_envelope(
    buffer: &[f32],
    sample_rate: usize,
    attack_time: f64,
    release_time: f64,
) -> Vec<f32> {
    let mut envelope = Vec::with_capacity(buffer.len());

    let attack_gain = (-1.0 / (sample_rate as f64 * attack_time)).exp() as f32;
    let release_gain = (-1.0 / (sample_rate as f64 * release_time)).exp() as f32;

    let mut envelope_output = 0.0;

    for sample in buffer {
        let envelope_input = (*sample).abs();

        envelope_output = if envelope_output < envelope_input {
            envelope_input + attack_gain * (envelope_output - envelope_input)
        } else {
            envelope_input + release_gain * (envelope_output - envelope_input)
        };

        envelope.push(envelope_output);
    }

    envelope
}

fn create_normalised_envelope(output_buffer: &[f32], sample_rate: usize) -> Vec<f32> {
    let attack_time = 0.01;
    let release_time = 0.01;

    let envelope = create_envelope(output_buffer, sample_rate, attack_time, release_time);

    let max_value = envelope
        .iter()
        .fold(0.0_f32, |current_max, value| current_max.max(*value));

    assert!(max_value > 0.0);

    envelope.iter().map(|value| *value / max_value).collect()
}

#[test]
fn test_gain_envelope() {
    let mut fixture = Fixture::default();

    for (gain, time_in_seconds) in [(0.0, 0.0), (1.0, 1.0), (0.0, 2.0)] {
        fixture
            .gain
            .gain
            .linear_ramp_to_value(gain, Timestamp::from_seconds(time_in_seconds));
    }

    let output_buffer = fixture.process_seconds(2.0);

    let envelope = create_normalised_envelope(
        output_buffer.get_data(SampleLocation::origin()),
        fixture.sample_rate,
    );

    for (frame, value) in envelope.iter().enumerate() {
        let time = Timestamp::from_samples(frame as f64, fixture.sample_rate).get_seconds();
        let expected_value = if time < 1.0 { time } else { 2.0 - time };
        assert_relative_eq!(expected_value as f32, value, epsilon = 0.05);
    }
}

#[test]
fn test_peak() {
    let mut fixture = Fixture::default();

    let gain = 0.5;

    fixture.gain.gain.set_value_at_time(gain, Timestamp::zero());

    let output_buffer = fixture.process_seconds(2.0);
    let output_samples = output_buffer.get_data(SampleLocation::origin());

    let max_value = output_samples.iter().fold(0.0_f32, |a, b| a.max(*b));
    let max_relative = Level::from_db(1.0).as_db() as f32;
    assert_relative_eq!(max_value, gain as f32, max_relative = max_relative);
}
