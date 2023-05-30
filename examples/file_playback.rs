#[path = "./lib/helpers.rs"]
mod helpers;

use std::{thread, time};

use rawdio::{
    create_engine_with_options, AudioBuffer, EngineOptions, Gain, Level, Sampler, Timestamp,
};
use structopt::StructOpt;

use helpers::{read_file_into_buffer, AudioCallback};

#[derive(Debug, StructOpt)]
struct Options {
    file_to_play: String,
}

fn main() {
    let options = Options::from_args();
    play_file(&options.file_to_play);
}

fn play_file(file_to_play: &str) {
    let sample = read_file_into_buffer(file_to_play);
    let sample_rate = sample.sample_rate();

    let (mut context, process) =
        create_engine_with_options(EngineOptions::default().with_sample_rate(sample_rate));

    let audio_callback = AudioCallback::new(process, sample_rate);

    let length_in_seconds = sample.length_in_seconds().ceil() as u64;
    let length_in_samples = sample.frame_count();
    let mut sampler = Sampler::new(context.as_ref(), sample);

    let channel_count = 2;
    let mut gain = Gain::new(context.as_ref(), channel_count);

    sampler.node.connect_to(&gain.node);
    sampler.start_now();
    sampler.enable_loop(
        Timestamp::zero(),
        Timestamp::from_samples(length_in_samples as f64, sample_rate),
    );

    gain.node.connect_to_output();

    gain.gain
        .set_value_at_time(Level::from_db(-6.0).as_linear(), Timestamp::zero());

    context.start();

    thread::sleep(time::Duration::from_secs(4 * length_in_seconds));

    context.stop();

    thread::sleep(time::Duration::from_secs(1));

    drop(audio_callback);
}
