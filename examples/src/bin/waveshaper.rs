use examples::AudioCallback;
use rawdio::{
    connect_nodes, create_engine_with_options, EngineOptions, Level, Oscillator, Timestamp,
    Waveshaper,
};
use std::{thread, time};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Options {
    #[structopt(long = "mix", default_value = "1")]
    mix: f64,
}

fn main() {
    let sample_rate = 44_100;

    let options = Options::from_args();

    let (mut context, process) =
        create_engine_with_options(EngineOptions::default().with_sample_rate(sample_rate));

    let _audio_callback = AudioCallback::new(process, sample_rate);

    let channel_count = 2;

    let frequency = 440.0;
    let oscillator = Oscillator::with_harmonics(
        context.as_ref(),
        frequency,
        channel_count,
        &[
            Level::from_db(-3.0),
            Level::from_db(-9.0),
            Level::from_db(-15.0),
        ],
    );

    let threshold = Level::from_db(-6.0);
    let mut shaper = Waveshaper::soft_saturator(context.as_ref(), channel_count, threshold);
    shaper.mix().set_value_now(options.mix);

    shaper.overdrive().set_value_at_time(0.0, Timestamp::zero());
    shaper
        .overdrive()
        .linear_ramp_to_value(1.0, Timestamp::zero(), Timestamp::from_seconds(5.0));

    shaper.overdrive().linear_ramp_to_value(
        0.0,
        Timestamp::from_seconds(5.0),
        Timestamp::from_seconds(10.0),
    );

    connect_nodes!(oscillator => shaper => "output");

    context.start();

    loop {
        context.process_notifications();
        thread::sleep(time::Duration::from_millis(10));
    }
}
