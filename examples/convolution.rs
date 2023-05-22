#[path = "./lib/helpers.rs"]
mod helpers;

use helpers::{read_file_into_buffer, render_audio_process_to_file};
use rawdio::{create_engine_with_options, AudioBuffer, Convolution, EngineOptions, Gain, Sampler};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Options {
    input: String,
    impulse: String,
    output: String,
}

fn main() {
    let options = Options::from_args();
    process_file(&options);
}

fn process_file(options: &Options) {
    let input = read_file_into_buffer(&options.input);
    let impulse = read_file_into_buffer(&options.impulse);

    let sample_rate = input.sample_rate();
    let input_channels = input.channel_count();
    let duration = input.duration() + impulse.duration();

    let (mut context, process) =
        create_engine_with_options(EngineOptions::default().with_sample_rate(sample_rate));

    let mut sample_player = Sampler::new(context.as_ref(), input);
    let convolution = Convolution::new(context.as_ref(), input_channels, impulse);
    let mut dry_gain = Gain::new(context.as_ref(), input_channels);
    let mut wet_gain = Gain::new(context.as_ref(), input_channels);
    let mut output = Gain::new(context.as_ref(), input_channels);

    wet_gain.gain.set_value_now(0.05);
    dry_gain.gain.set_value_now(1.0);
    output.gain.set_value_now(0.5);

    sample_player.node.connect_to(&dry_gain.node);
    sample_player.node.connect_to(&convolution.node);
    dry_gain.node.connect_to(&output.node);
    convolution.node.connect_to(&wet_gain.node);
    wet_gain.node.connect_to(&output.node);
    output.node.connect_to_output();

    sample_player.start_now();
    context.start();

    render_audio_process_to_file(sample_rate, &options.output, process, duration);

    context.stop();
}
