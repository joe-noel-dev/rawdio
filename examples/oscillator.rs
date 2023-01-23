use std::{thread, time};

use rawdio::{create_engine, Context, Gain, Level, Mixer, Oscillator, Timestamp};
use utilities::AudioCallback;

fn main() {
    let sample_rate = 44100;
    let (mut context, audio_process) = create_engine(sample_rate);
    let audio_callback = AudioCallback::new(audio_process, sample_rate);

    let mut oscillators = create_oscillators(context.as_ref());
    let mut gain = create_gain(context.as_ref());
    let mut mixer = create_mixer(context.as_ref());

    schedule_gain_changes(&mut gain);
    make_connections(&mut oscillators, &mut gain, &mut mixer);

    run(context.as_mut());

    drop(audio_callback);
}

fn create_oscillators(context: &dyn Context) -> [Oscillator; 4] {
    let channel_count = 1;

    [
        (440.0, Level::from_db(-3.0)),
        (880.0, Level::from_db(-9.0)),
        (1320.0, Level::from_db(-15.0)),
        (1760.0, Level::from_db(-21.0)),
    ]
    .map(|(frequency, level)| {
        let mut oscillator = Oscillator::new(context.get_command_queue(), frequency, channel_count);

        oscillator
            .gain
            .set_value_at_time(level.as_gain(), Timestamp::zero());

        oscillator
            .frequency
            .exponential_ramp_to_value(4.0 * frequency, Timestamp::from_seconds(4.0));

        oscillator
    })
}

fn create_mixer(context: &dyn Context) -> Mixer {
    Mixer::mono_to_stereo_splitter(context.get_command_queue())
}

fn create_gain(context: &dyn Context) -> Gain {
    let gain_channel_count = 1;
    Gain::new(context.get_command_queue(), gain_channel_count)
}

fn schedule_gain_changes(gain: &mut Gain) {
    gain.gain.set_value_at_time(0.0, Timestamp::zero());

    gain.gain
        .linear_ramp_to_value(1.0, Timestamp::from_seconds(0.1));

    gain.gain
        .linear_ramp_to_value(0.0, Timestamp::from_seconds(4.0));
}

fn make_connections(oscillators: &mut [Oscillator], gain: &mut Gain, mixer: &mut Mixer) {
    for oscillator in oscillators {
        oscillator.node.connect_to(&gain.node);
    }

    gain.node.connect_to(&mixer.node);

    mixer.node.connect_to_output();
}

fn run(context: &mut dyn Context) {
    let process_duration = time::Duration::from_secs(4);
    let post_process_duration = time::Duration::from_secs(1);

    context.start();
    thread::sleep(process_duration);
    context.stop();
    thread::sleep(post_process_duration);
}
