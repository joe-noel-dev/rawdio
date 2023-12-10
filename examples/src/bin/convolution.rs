use examples::{read_file_into_buffer, render_audio_process_to_file};
use rawdio::{create_engine_with_options, AudioBuffer, Convolution, EngineOptions, Gain, Sampler};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Options {
    #[structopt(short, long)]
    input: String,

    #[structopt(short = "m", long)]
    impulse: String,

    #[structopt(short, long)]
    output: String,

    #[structopt(short, long, default_value = "1.0")]
    wet: f64,

    #[structopt(short, long, default_value = "0.0")]
    dry: f64,
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
    let mut convolution = Convolution::new(context.as_ref(), input_channels, impulse);

    convolution.dry.set_value_now(options.dry);
    convolution.wet.set_value_now(options.wet);

    let mut output = Gain::new(context.as_ref(), input_channels);
    output.gain.set_value_now(1.0);

    sample_player.node.connect_to(&convolution.node);
    convolution.node.connect_to(&output.node);
    output.node.connect_to_output();

    sample_player.start_now();
    context.start();

    render_audio_process_to_file(sample_rate, &options.output, process, duration);

    context.stop();
}
