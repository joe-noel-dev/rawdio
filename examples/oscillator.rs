use std::{thread, time};

use rust_audio_engine::{context::Context, osc::oscillator::Oscillator};

use crate::audio_callback::AudioCallback;

#[path = "./lib/audio_callback.rs"]
mod audio_callback;

fn main() {
    let mut context = Context::new(44100);
    let _audio_callack = AudioCallback::new(context.get_audio_process());

    {
        let mut oscillator = Oscillator::new(context.get_command_queue(), 440.0);

        context.connect_to_output(&oscillator);
        context.start();
        thread::sleep(time::Duration::from_secs(3));

        context.process_notifications();

        oscillator
            .frequency
            .linear_ramp_to_value(220.0, context.current_time().incremented_by_seconds(3.0));
        thread::sleep(time::Duration::from_secs(4));

        context.process_notifications();
        context.stop();
    }

    thread::sleep(time::Duration::from_secs(1));
}
