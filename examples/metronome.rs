use std::time::Duration;

use rawdio::{create_engine, Adsr, Context, Oscillator, Timestamp};
use structopt::StructOpt;

use utilities::AudioCallback;

#[derive(StructOpt)]
struct Options {
    #[structopt(long = "tempo", default_value = "120")]
    tempo: f64,
}

fn schedule_events(
    oscillator: &mut Oscillator,
    adsr: &mut Adsr,
    beat_interval: f64,
    from_time: Timestamp,
    to_time: Timestamp,
) {
    let start = from_time.as_seconds() / beat_interval;
    let mut beat = start.ceil() as usize;

    let low_frequency = 440.0;
    let high_frequency = 880.0;

    loop {
        let beat_position = Timestamp::from_seconds(beat as f64 * beat_interval);
        if beat_position >= to_time {
            break;
        }

        let frequency = if beat % 4 == 0 {
            high_frequency
        } else {
            low_frequency
        };

        oscillator
            .frequency
            .set_value_at_time(frequency, beat_position);

        let note_duration = 0.1;
        adsr.note_on_at_time(beat_position);
        adsr.note_off_at_time(beat_position.incremented_by_seconds(note_duration));

        beat += 1;
    }
}

fn main() {
    let options = Options::from_args();
    let tempo = options.tempo;

    let sample_rate = 44_100;

    let (mut context, audio_process) = create_engine(sample_rate);
    let _callback = AudioCallback::new(audio_process, sample_rate);

    let channel_count = 2;
    let mut oscillator = create_oscillator(context.as_ref(), channel_count);
    let mut adsr = create_adsr(context.as_ref(), channel_count, sample_rate);

    oscillator.node.connect_to(&adsr.node);
    adsr.node.connect_to_output();

    context.start();

    let mut last_sequence_time = Timestamp::zero();
    let process_interval = Duration::from_secs_f64(1.0 / 60.0);
    let look_ahead_interval = 2 * process_interval;
    let beat_interval = 60.0 / tempo;
    loop {
        let next_interval_start = context
            .current_time()
            .incremented_by_seconds(look_ahead_interval.as_secs_f64());

        schedule_events(
            &mut oscillator,
            &mut adsr,
            beat_interval,
            last_sequence_time,
            next_interval_start,
        );

        context.process_notifications();

        last_sequence_time = next_interval_start;

        std::thread::sleep(process_interval);
    }
}

fn create_oscillator(context: &dyn Context, channel_count: usize) -> Oscillator {
    let osc_frequency = 440.0;
    Oscillator::sine(context, osc_frequency, channel_count)
}

fn create_adsr(context: &dyn Context, channel_count: usize, sample_rate: usize) -> Adsr {
    let mut adsr = Adsr::new(context, channel_count, sample_rate);

    adsr.set_attack_time(Duration::from_millis(20));
    adsr.set_release_time(Duration::from_millis(100));

    adsr
}
