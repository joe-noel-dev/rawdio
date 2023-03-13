use rawdio::{create_engine, AudioBuffer, Convolution, Sampler};
use structopt::StructOpt;
use utilities::{read_file_into_buffer, render_audio_process_to_file};

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

    println!("Total duration = {}", duration.as_secs_f64());

    let (mut context, audio_process) = create_engine(sample_rate);

    let mut sample_player = Sampler::new(context.as_ref(), sample_rate, input);
    let convolution = Convolution::new(context.as_ref(), input_channels, impulse);

    sample_player.node.connect_to(&convolution.node);
    convolution.node.connect_to_output();

    sample_player.start_now();
    context.start();

    render_audio_process_to_file(sample_rate, &options.output, audio_process, duration);

    context.stop();
}
