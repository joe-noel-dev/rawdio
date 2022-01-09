use std::{thread, time};

use rust_audio_engine::{
    context::Context, nodes::oscillator::OscillatorNode, timestamp::Timestamp,
};

use crate::audio_callback::AudioCallback;

#[path = "./lib/audio_callback.rs"]
mod audio_callback;

fn main() {
    let mut context = Context::new(44100);
    let _audio_callack = AudioCallback::new(context.get_audio_process());

    {
        let mut oscillator = OscillatorNode::new(context.get_command_queue(), 432.0);
        oscillator
            .gain
            .set_value_immediate(0.5, Timestamp::from_seconds(0.0));

        oscillator
            .gain
            .linear_ramp_to_value(0.6, Timestamp::from_seconds(2.0));

        oscillator
            .gain
            .linear_ramp_to_value(0.0, Timestamp::from_seconds(4.0));

        context.connect_to_output(&oscillator);
        context.start();

        thread::sleep(time::Duration::from_secs(4));

        context.process_notifications();
        context.stop();
    }

    thread::sleep(time::Duration::from_secs(1));
}
