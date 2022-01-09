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
    let mut context = Context::new(44100);
    let _audio_callack = AudioCallback::new(context.get_audio_process());

    {
        let oscillator = OscillatorNode::new(context.get_command_queue(), 440.0);
        let mut gain = GainNode::new(context.get_command_queue());

        oscillator.connect_to(gain.get_id());
        gain.connect_to_output();

        gain.gain
            .set_value_at_time(0.5, Timestamp::from_seconds(0.0));

        gain.gain
            .linear_ramp_to_value(0.9, Timestamp::from_seconds(2.0));

        gain.gain
            .linear_ramp_to_value(0.0, Timestamp::from_seconds(4.0));

        context.start();

        thread::sleep(time::Duration::from_secs(4));

        context.process_notifications();
        context.stop();
    }

    thread::sleep(time::Duration::from_secs(1));
}
