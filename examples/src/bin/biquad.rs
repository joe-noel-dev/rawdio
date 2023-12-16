use std::{thread, time::Duration};

use examples::{read_file_into_buffer, AudioCallback};

use rawdio::{
    connect_nodes, create_engine_with_options, AudioBuffer, Biquad, BiquadFilterType,
    EngineOptions, Sampler, Timestamp,
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
    let mut biquad = Biquad::new(context.as_ref(), channel_count, BiquadFilterType::LowPass);

    biquad.frequency().set_value_now(20.0);

    biquad.frequency().exponential_ramp_to_value(
        20_000.0,
        Timestamp::zero(),
        Timestamp::from_seconds(10.0),
    );

    biquad.frequency().exponential_ramp_to_value(
        20.0,
        Timestamp::from_seconds(10.0),
        Timestamp::from_seconds(20.0),
    );

    sampler.start_now();
    sampler.enable_loop(
        Timestamp::zero(),
        Timestamp::from_samples(length_in_samples as f64, sample_rate),
    );

    connect_nodes!(sampler => biquad => "output");

    context.start();

    loop {
        thread::sleep(Duration::from_secs(1));
        context.process_notifications();
    }
}
