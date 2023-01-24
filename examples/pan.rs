use std::{thread, time};

use rawdio::{create_engine, Context, Level, Oscillator, Pan, Timestamp};

use utilities::AudioCallback;

fn main() {
    let sample_rate = 44100;
    let (mut context, audio_process) = create_engine(sample_rate);
    let audio_callack = AudioCallback::new(audio_process, sample_rate);

    let oscillator = create_oscillator(context.as_ref());
    let pan = create_pan(context.as_ref());

    oscillator.node.connect_to(&pan.node);
    pan.node.connect_to_output();

    context.start();
    thread::sleep(time::Duration::from_secs(5));
    context.stop();

    thread::sleep(time::Duration::from_secs(1));

    drop(audio_callack);
}

fn create_oscillator(context: &dyn Context) -> Oscillator {
    let frequency = 440.0;

    let channel_count = 2;

    let mut oscillator = Oscillator::sine(context, frequency, channel_count);

    oscillator
        .gain
        .set_value_at_time(Level::from_db(-3.0).as_gain(), Timestamp::zero());

    oscillator
}

fn create_pan(context: &dyn Context) -> Pan {
    let pan_input_count = 2;
    let mut pan = Pan::new(context, pan_input_count);

    pan.pan.set_value_at_time(-1.0, Timestamp::zero());

    pan.pan
        .linear_ramp_to_value(1.0, Timestamp::zero(), Timestamp::from_seconds(4.0));

    pan
}
