use std::{thread, time};

use rust_audio_engine::{
    context::Context,
    graph::node::Node,
    nodes::{gain::GainNode, oscillator::OscillatorNode},
    timestamp::Timestamp,
};

use crate::audio_callback::AudioCallback;

#[path = "./lib/audio_callback.rs"]
mod audio_callback;

fn main() {
    let sample_rate = 44100;
    let mut context = Context::new(sample_rate);
    let _audio_callack = AudioCallback::new(context.get_audio_process(), sample_rate);

    let mut oscillator_1 = OscillatorNode::new(context.get_command_queue(), 440.0);
    oscillator_1
        .gain
        .set_value_at_time(0.8, Timestamp::from_seconds(0.0));

    let mut oscillator_2 = OscillatorNode::new(context.get_command_queue(), 880.0);
    oscillator_2
        .gain
        .set_value_at_time(0.4, Timestamp::from_seconds(0.0));

    let mut oscillator_3 = OscillatorNode::new(context.get_command_queue(), 1320.0);
    oscillator_3
        .gain
        .set_value_at_time(0.2, Timestamp::from_seconds(0.0));

    let mut oscillator_4 = OscillatorNode::new(context.get_command_queue(), 1760.0);
    oscillator_4
        .gain
        .set_value_at_time(0.1, Timestamp::from_seconds(0.0));

    let mut gain = GainNode::new(context.get_command_queue());

    oscillator_1.connect_to(gain.get_id());
    oscillator_2.connect_to(gain.get_id());
    oscillator_3.connect_to(gain.get_id());
    oscillator_4.connect_to(gain.get_id());

    gain.connect_to_output();

    gain.gain
        .set_value_at_time(0.9, Timestamp::from_seconds(0.0));

    gain.gain
        .linear_ramp_to_value(0.0, Timestamp::from_seconds(4.0));

    context.start();

    thread::sleep(time::Duration::from_secs(4));

    context.process_notifications();
    context.stop();

    thread::sleep(time::Duration::from_secs(1));
}
