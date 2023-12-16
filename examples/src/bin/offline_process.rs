use examples::render_audio_process_to_file;
use rawdio::{prelude::*, Gain, Oscillator, Pan};
use std::time::Duration;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Options {
    output_file: String,
}

fn main() {
    let options = Options::from_args();
    render_file(&options.output_file);
}

fn render_file(output_file: &str) {
    let sample_rate = 44100;

    let (mut context, process) =
        create_engine_with_options(EngineOptions::default().with_sample_rate(sample_rate));

    let oscillators = create_oscillators(context.as_ref());
    let gain = create_gain(context.as_ref());
    let pan = create_pan(context.as_ref());

    make_connections(&oscillators, &gain, &pan);

    context.start();

    render_audio_process_to_file(sample_rate, output_file, process, Duration::from_secs(4));

    context.stop();
}

fn create_oscillators(context: &dyn Context) -> [Oscillator; 4] {
    let output_count = 2;

    [(440.0, 0.4), (880.0, 0.2), (1320.0, 0.1), (1760.0, 0.05)].map(|(frequency, gain)| {
        let mut oscillator = Oscillator::sine(context, frequency, output_count);

        oscillator.gain().set_value_at_time(gain, Timestamp::zero());

        oscillator
    })
}

fn create_gain(context: &dyn Context) -> Gain {
    let channel_count = 2;

    let mut gain = Gain::new(context, channel_count);
    gain.gain().set_value_at_time(0.9, Timestamp::zero());
    gain
}

fn create_pan(context: &dyn Context) -> Pan {
    let pan_input_count = 2;
    let mut pan = Pan::new(context, pan_input_count);

    pan.pan().set_value_at_time(-1.0, Timestamp::zero());
    pan.pan()
        .linear_ramp_to_value(1.0, Timestamp::zero(), Timestamp::from_seconds(2.0));
    pan.pan().linear_ramp_to_value(
        -1.0,
        Timestamp::from_seconds(2.0),
        Timestamp::from_seconds(4.0),
    );

    pan
}

fn make_connections(oscillators: &[Oscillator], gain: &Gain, pan: &Pan) {
    for oscillator in oscillators {
        connect_nodes!(oscillator => gain);
    }

    connect_nodes!(gain => pan => "output");
}
