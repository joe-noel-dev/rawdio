use std::time::Duration;

use itertools::Itertools;
use rawdio::{
    create_engine, AudioBuffer, AudioProcess, Context, OwnedAudioBuffer, Pan, SampleLocation,
    Timestamp,
};

struct Fixture {
    sample_rate: usize,
    channel_count: usize,
    _context: Box<dyn Context>,
    audio_process: Box<dyn AudioProcess>,
    pan: Pan,
}

fn make_noise_buffer(
    frame_count: usize,
    channel_count: usize,
    sample_rate: usize,
) -> OwnedAudioBuffer {
    let reference = OwnedAudioBuffer::white_noise(frame_count, 1, sample_rate);

    let mut buffer = OwnedAudioBuffer::new(frame_count, channel_count, sample_rate);

    (0..channel_count).for_each(|channel| {
        buffer.copy_from(
            &reference,
            SampleLocation::origin(),
            SampleLocation::channel(channel),
            1,
            reference.frame_count(),
        );
    });

    buffer
}

impl Fixture {
    fn process(&mut self, duration: Duration) -> OwnedAudioBuffer {
        let frame_count = (duration.as_secs_f64() * self.sample_rate as f64).ceil() as usize;
        let input_buffer = make_noise_buffer(frame_count, self.channel_count, self.sample_rate);
        let mut output_buffer =
            OwnedAudioBuffer::new(frame_count, self.channel_count, self.sample_rate);
        self.audio_process
            .process(&input_buffer, &mut output_buffer);
        output_buffer
    }
}

impl Default for Fixture {
    fn default() -> Self {
        let sample_rate = 48_000;
        let (mut context, audio_process) = create_engine(sample_rate);

        let channel_count = 2;

        let pan = Pan::new(context.as_ref(), channel_count);

        pan.node.connect_to_input();
        pan.node.connect_to_output();

        context.start();

        Self {
            _context: context,
            audio_process,
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
