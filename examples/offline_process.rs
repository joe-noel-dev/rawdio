use file_utils::render_audio_process_to_file;
use rawdio::{create_engine, Context, Gain, Oscillator, Pan, Timestamp};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Options {
    output_file: String,
}

#[path = "./lib/file_utils.rs"]
mod file_utils;

fn main() {
    let options = Options::from_args();
    render_file(&options.output_file);
}

fn render_file(output_file: &str) {
    let sample_rate = 44100;
    let (mut context, mut audio_process) = create_engine(sample_rate);

    let mut oscillators = create_oscillators(context.as_ref());
    let mut gain = create_gain(context.as_ref());
    let mut pan = create_pan(context.as_ref());

    make_connections(&mut oscillators, &mut gain, &mut pan);

    context.start();

    render_audio_process_to_file(sample_rate, output_file, audio_process.as_mut());
    context.stop();
}

fn create_oscillators(context: &dyn Context) -> [Oscillator; 4] {
    let output_count = 2;

    [(440.0, 0.4), (880.0, 0.2), (1320.0, 0.1), (1760.0, 0.05)].map(|(frequency, gain)| {
        let mut oscillator = Oscillator::new(context.get_command_queue(), frequency, output_count);
        oscillator.gain.set_value_at_time(gain, Timestamp::zero());
        oscillator
    })
}

fn create_gain(context: &dyn Context) -> Gain {
    let channel_count = 2;

    let mut gain = Gain::new(context.get_command_queue(), channel_count);
    gain.gain.set_value_at_time(0.9, Timestamp::zero());
    gain
}

fn create_pan(context: &dyn Context) -> Pan {
    let pan_input_count = 2;
    let mut pan = Pan::new(context.get_command_queue(), pan_input_count);

    pan.pan.set_value_at_time(-1.0, Timestamp::zero());
    pan.pan
        .linear_ramp_to_value(1.0, Timestamp::from_seconds(2.0));
    pan.pan
        .linear_ramp_to_value(-1.0, Timestamp::from_seconds(4.0));

    pan
}

fn make_connections(oscillators: &mut [Oscillator], gain: &mut Gain, pan: &mut Pan) {
    for oscillator in oscillators {
        oscillator.node.connect_to(&gain.node);
    }

    gain.node.connect_to(&pan.node);
    pan.node.connect_to_output();
}
