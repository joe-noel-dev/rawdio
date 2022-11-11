use std::{thread, time};

use rust_audio_engine::{create_context, Level, Node, Oscillator, Pan, Timestamp};

use crate::audio_callback::AudioCallback;

#[path = "./lib/audio_callback.rs"]
mod audio_callback;

fn main() {
    let sample_rate = 44100;
    let mut context = create_context(sample_rate);
    let _audio_callack = AudioCallback::new(context.get_audio_process(), sample_rate);

    let frequency = 440.0;
    let mut oscillator = Oscillator::new(context.get_command_queue(), frequency);
    oscillator
        .gain
        .set_value_at_time(Level::from_db(-3.0).as_gain(), Timestamp::zero());

    let pan_input_count = 1;
    let mut pan = Pan::new(context.get_command_queue(), pan_input_count);

    oscillator.connect_to(pan.get_id());
    pan.connect_to_output();

    pan.pan.set_value_at_time(-1.0, Timestamp::zero());

    pan.pan
        .linear_ramp_to_value(1.0, Timestamp::from_seconds(4.0));

    context.start();

    thread::sleep(time::Duration::from_secs(5));

    context.stop();

    thread::sleep(time::Duration::from_secs(1));
}
