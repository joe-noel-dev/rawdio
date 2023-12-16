use std::time::Duration;

use examples::{read_file_into_buffer, AudioCallback};
use rawdio::{
    connect_nodes, create_engine_with_options, AudioBuffer, Compressor, EngineOptions, Sampler,
    Timestamp,
};
use structopt::StructOpt;

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

    let _audio_callback = AudioCallback::new(process, sample_rate);

    let length_in_samples = sample.frame_count();
    let mut sampler = Sampler::new(context.as_ref(), sample);

    let channel_count = 2;

    let mut compressor = Compressor::new(context.as_ref(), channel_count);

    compressor.threshold().set_value_now(-18.0);
    compressor.attack().set_value_now(1.0);
    compressor.release().set_value_now(10.0);
    compressor.wet().set_value_now(0.75);
    compressor.dry().set_value_now(0.25);
    compressor.knee().set_value_now(6.0);
    compressor.ratio().set_value_now(4.0);

    sampler.start_now();
    sampler.enable_loop(
        Timestamp::zero(),
        Timestamp::from_samples(length_in_samples as f64, sample_rate),
    );

    connect_nodes!(sampler => compressor);

    context.start();

    loop {
        std::thread::sleep(Duration::from_secs(1));
        context.process_notifications();
    }
}
