use examples::AudioCallback;
use rawdio::{prelude::*, Gain, Mixer, Oscillator};
use std::{thread, time};

fn main() {
    let sample_rate = 44100;
    let (mut context, process) =
        create_engine_with_options(EngineOptions::default().with_sample_rate(sample_rate));

    let audio_callback = AudioCallback::new(process, sample_rate);

    let oscillator = create_oscillator(context.as_ref());
    let mut gain = create_gain(context.as_ref());
    let mixer = create_mixer(context.as_ref());

    schedule_gain_changes(&mut gain);

    connect_nodes!(
        oscillator => gain => mixer => "output"
    );

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
        .gain()
        .set_value_at_time(Level::unity().as_linear(), Timestamp::zero());

    oscillator.frequency().exponential_ramp_to_value(
        100.0 * frequency,
        Timestamp::zero(),
        Timestamp::from_seconds(4.0),
    );

    oscillator
}

fn create_mixer(context: &dyn Context) -> Mixer {
    Mixer::mono_to_stereo_splitter(context)
}

fn create_gain(context: &dyn Context) -> Gain {
    let gain_channel_count = 1;
    Gain::new(context, gain_channel_count)
}

fn schedule_gain_changes(gain: &mut Gain) {
    gain.gain().set_value_at_time(0.0, Timestamp::zero());

    gain.gain()
        .linear_ramp_to_value(1.0, Timestamp::zero(), Timestamp::from_seconds(0.1));

    gain.gain().linear_ramp_to_value(
        0.0,
        Timestamp::from_seconds(3.9),
        Timestamp::from_seconds(4.0),
    );
}

fn run(context: &mut dyn Context) {
    let process_duration = time::Duration::from_secs(4);
    let post_process_duration = time::Duration::from_secs(1);

    context.start();
    thread::sleep(process_duration);
    context.stop();
    thread::sleep(post_process_duration);
}
