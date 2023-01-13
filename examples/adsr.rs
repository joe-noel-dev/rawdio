use std::{
    thread,
    time::{self, Duration},
};

use rawdio::{create_engine, Adsr, Context, Gain, Level, Oscillator, Splitter, Timestamp};

use crate::audio_callback::AudioCallback;

#[path = "./lib/audio_callback.rs"]
mod audio_callback;

fn main() {
    let sample_rate = 44100;
    let (mut context, audio_process) = create_engine(sample_rate);
    let _audio_callack = AudioCallback::new(audio_process, sample_rate);

    let mut oscillators = create_oscillators(context.as_ref());
    let mut adsr = create_adsr(context.as_ref());
    let mut gain = create_gain(context.as_ref());
    let mut splitter = create_splitter(context.as_ref());

    schedule_notes(&mut adsr);
    make_connections(&mut oscillators, &mut adsr, &mut gain, &mut splitter);

    run(context.as_mut());
}

fn create_oscillators(context: &dyn Context) -> [Oscillator; 4] {
    let channel_count = 1;

    [
        (440.0, Level::from_db(-3.0)),
        (880.0, Level::from_db(-9.0)),
        (1320.0, Level::from_db(-15.0)),
        (1760.0, Level::from_db(-21.0)),
    ]
    .map(|(frequency, level)| {
        let mut oscillator = Oscillator::new(context.get_command_queue(), frequency, channel_count);

        oscillator
            .gain
            .set_value_at_time(level.as_gain(), Timestamp::zero());

        oscillator
    })
}

fn create_splitter(context: &dyn Context) -> Splitter {
    let splitter_input_channel_count = 1;
    let splitter_output_channel_count = 2;
    Splitter::new(
        context.get_command_queue(),
        splitter_input_channel_count,
        splitter_output_channel_count,
    )
}

fn create_gain(context: &dyn Context) -> Gain {
    let gain_channel_count = 1;
    Gain::new(context.get_command_queue(), gain_channel_count)
}

fn create_adsr(context: &dyn Context) -> Adsr {
    let adsr_channel_count = 1;
    let mut adsr = Adsr::new(
        context.get_command_queue(),
        adsr_channel_count,
        context.get_sample_rate(),
    );

    adsr.set_attack_time(Duration::from_millis(100));
    adsr.set_decay_time(Duration::from_millis(150));
    adsr.set_sustain_level(Level::from_db(-12.0));
    adsr.set_release_time(Duration::from_millis(200));

    adsr
}

fn schedule_notes(adsr: &mut Adsr) {
    adsr.note_on_at_time(Timestamp::zero());
    adsr.note_off_at_time(Timestamp::from_seconds(0.5));

    adsr.note_on_at_time(Timestamp::from_seconds(1.0));
    adsr.note_off_at_time(Timestamp::from_seconds(1.5));

    adsr.note_on_at_time(Timestamp::from_seconds(2.0));
    adsr.note_off_at_time(Timestamp::from_seconds(2.5));

    adsr.note_on_at_time(Timestamp::from_seconds(3.0));
    adsr.note_off_at_time(Timestamp::from_seconds(3.5));
}

fn make_connections(
    oscillators: &mut [Oscillator],
    adsr: &mut Adsr,
    gain: &mut Gain,
    splitter: &mut Splitter,
) {
    for oscillator in oscillators {
        oscillator.node.connect_to(&adsr.node);
    }

    adsr.node.connect_to(&gain.node);

    gain.node.connect_to(&splitter.node);

    splitter.node.connect_to_output();
}

fn run(context: &mut dyn Context) {
    let process_duration = time::Duration::from_secs(4);
    let post_process_duration = time::Duration::from_secs(1);

    context.start();
    thread::sleep(process_duration);
    context.stop();
    thread::sleep(post_process_duration);
}
