use std::time::Duration;

use itertools::Itertools;
use rawdio::{
    create_engine, AudioBuffer, AudioProcess, Context, Oscillator, OwnedAudioBuffer, Pan,
    SampleLocation, Timestamp,
};

struct Fixture {
    sample_rate: usize,
    channel_count: usize,
    _context: Box<dyn Context>,
    audio_process: Box<dyn AudioProcess>,
    _oscillator: Oscillator,
    pan: Pan,
}

impl Fixture {
    fn process(&mut self, duration: Duration) -> OwnedAudioBuffer {
        let frame_count = (duration.as_secs_f64() * self.sample_rate as f64).ceil() as usize;
        let mut output_buffer =
            OwnedAudioBuffer::new(frame_count, self.channel_count, self.sample_rate);
        self.audio_process.process(&mut output_buffer);
        output_buffer
    }
}

impl Default for Fixture {
    fn default() -> Self {
        let sample_rate = 48_000;
        let (mut context, audio_process) = create_engine(sample_rate);

        let oscillator_frequency = 997.0;
        let channel_count = 2;
        let oscillator = Oscillator::new(
            context.get_command_queue(),
            oscillator_frequency,
            channel_count,
        );

        let pan = Pan::new(context.get_command_queue(), channel_count);

        oscillator.node.connect_to(&pan.node);
        pan.node.connect_to_output();

        context.start();

        Self {
            _context: context,
            audio_process,
            _oscillator: oscillator,
            pan,
            sample_rate,
            channel_count,
        }
    }
}

fn get_energy_of_channel(audio_buffer: &dyn AudioBuffer, channel_index: usize) -> f64 {
    let data = audio_buffer.get_channel_data(SampleLocation::new(channel_index, 0));
    data.iter().fold(0.0_f64, |total_energy, sample| {
        total_energy + (*sample).powf(2.0) as f64
    })
}

fn process_with_pan(pan: f64) -> Vec<f64> {
    let mut fixture = Fixture::default();

    fixture.pan.pan.set_value_at_time(pan, Timestamp::zero());

    let audio = fixture.process(Duration::from_secs_f64(1.0));

    (0..audio.channel_count())
        .map(|channel| get_energy_of_channel(&audio, channel))
        .collect()
}

#[test]
fn panned_fully_left() {
    let energy = process_with_pan(-1.0);

    assert!(energy[0] > 0.0);
    assert!(energy[1] == 0.0);
}

#[test]
fn panned_fully_right() {
    let energy = process_with_pan(1.0);

    assert!(energy[0] == 0.0);
    assert!(energy[1] > 0.0);
}

#[test]
fn panned_centrally() {
    let energy = process_with_pan(0.0);
    assert!(energy.iter().all_equal());
}

#[test]
fn panned_part_left() {
    let energy = process_with_pan(-0.5);

    assert!(energy[0] > 0.0);
    assert!(energy[1] > 0.0);
    assert!(energy[0] > energy[1]);
}

#[test]
fn panned_part_right() {
    let energy = process_with_pan(0.5);

    assert!(energy[0] > 0.0);
    assert!(energy[1] > 0.0);
    assert!(energy[0] < energy[1]);
}
