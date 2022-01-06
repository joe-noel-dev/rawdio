use std::{thread, time};

use rust_audio_engine::{context::Context, sources::oscillator::OscillatorType};

use crate::audio_callback::AudioCallback;

#[path = "./lib/audio_callback.rs"]
mod audio_callback;

fn main() {
    let mut context = Context::new(44100);
    let _audio_callack = AudioCallback::new(context.get_audio_process());

    let oscillator = context
        .add_oscillator()
        .with_type(OscillatorType::Sine)
        .with_frequency(440.0);

    context.connect_to_output(&oscillator);
    context.start();
    thread::sleep(time::Duration::from_secs(3));
    context.remove_oscillator(&oscillator);
    thread::sleep(time::Duration::from_secs(1));
    context.process_notifications();
    context.stop();
}
