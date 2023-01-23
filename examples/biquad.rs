use std::{thread, time::Duration};

use rawdio::{create_engine, AudioBuffer, Biquad, BiquadFilterType, Sampler, Timestamp};
use structopt::StructOpt;

use utilities::{read_file_into_buffer, AudioCallback};

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

    let (mut context, audio_process) = create_engine(sample_rate);
    let _audio_callack = AudioCallback::new(audio_process, sample_rate);

    let length_in_seconds = sample.length_in_seconds().ceil() as u64;
    let length_in_samples = sample.frame_count();
    let mut sampler = Sampler::new(context.get_command_queue(), sample_rate, sample);

    let channel_count = 2;
    let mut biquad = Biquad::new(context.as_ref(), channel_count, BiquadFilterType::LowPass);

    biquad.frequency.set_value_now(20.0);

    biquad
        .frequency
        .exponential_ramp_to_value(20_000.0, Timestamp::from_seconds(10.0));

    biquad
        .frequency
        .exponential_ramp_to_value(20.0, Timestamp::from_seconds(20.0));

    sampler.node.connect_to(&biquad.node);

    sampler.start_now();
    sampler.enable_loop(
        Timestamp::zero(),
        Timestamp::from_samples(length_in_samples as f64, sample_rate),
    );

    biquad.node.connect_to_output();

    context.start();

    thread::sleep(Duration::from_secs(4 * length_in_seconds));

    context.stop();

    thread::sleep(Duration::from_secs(1));
}
