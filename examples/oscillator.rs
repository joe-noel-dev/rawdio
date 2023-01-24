use std::{thread, time};

use rawdio::{create_engine, Context, Gain, Level, Mixer, Oscillator, Timestamp};
use utilities::AudioCallback;

fn main() {
    let sample_rate = 44100;
    let (mut context, audio_process) = create_engine(sample_rate);
    let audio_callback = AudioCallback::new(audio_process, sample_rate);

    let mut oscillator = create_oscillator(context.as_ref());
    let mut gain = create_gain(context.as_ref());
    let mut mixer = create_mixer(context.as_ref());

    schedule_gain_changes(&mut gain);
    make_connections(&mut oscillator, &mut gain, &mut mixer);

    run(context.as_mut());

    drop(audio_callback);
}

fn create_oscillator(context: &dyn Context) -> Oscillator {
    let channel_count = 1;

    let harmonics = [
        Level::from_db(-3.0),
        Level::from_db(-9.0),
        Level::from_db(-15.0),
        Level::from_db(-21.0),
        Level::from_db(-27.0),
    ];

    let frequency = 20.0;

    let mut oscillator = Oscillator::with_harmonics(context, 20.0, channel_count, &harmonics);

    oscillator
        .gain
        .set_value_at_time(Level::unity().as_gain(), Timestamp::zero());

    oscillator
        .frequency
        .exponential_ramp_to_value(100.0 * frequency, Timestamp::from_seconds(4.0));

    oscillator
}

fn create_mixer(context: &dyn Context) -> Mixer {
    Mixer::mono_to_stereo_splitter(context.get_command_queue())
}

fn create_gain(context: &dyn Context) -> Gain {
    let gain_channel_count = 1;
    Gain::new(context, gain_channel_count)
}

fn schedule_gain_changes(gain: &mut Gain) {
    gain.gain.set_value_at_time(0.0, Timestamp::zero());

    gain.gain
        .linear_ramp_to_value(1.0, Timestamp::from_seconds(0.1));

    gain.gain
        .set_value_at_time(1.0, Timestamp::from_seconds(3.9));

    gain.gain
        .linear_ramp_to_value(0.0, Timestamp::from_seconds(4.0));
}

fn make_connections(oscillator: &mut Oscillator, gain: &mut Gain, mixer: &mut Mixer) {
    oscillator.node.connect_to(&gain.node);
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
