use std::{cell::RefCell, rc::Rc, thread, time::Duration};

use rawdio::{
    create_engine, Context, Gain, Level, Mixer, Oscillator, OwnedAudioBuffer, Recorder, Timestamp,
};
use structopt::StructOpt;
use utilities::{write_buffer_into_file, AudioCallback};

#[derive(Debug, StructOpt)]
struct Options {
    output_file: String,
}

fn main() {
    let output_file = &Options::from_args().output_file;

    let sample_rate = 44100;
    let (mut context, audio_process) = create_engine(sample_rate);
    let _audio_callack = AudioCallback::new(audio_process, sample_rate);

    let mut oscillators = create_oscillators(context.as_ref());
    let mut gain = create_gain(context.as_ref());
    let mut mixer = create_mixer(context.as_ref());
    let recorder = create_recorder(context.as_mut());

    let process_duration = Timestamp::from_seconds(4.0);

    recorder.borrow_mut().record_now();
    recorder.borrow_mut().stop_record_at_time(process_duration);

    {
        let mut recorder = recorder.borrow_mut();
        make_connections(&mut oscillators, &mut gain, &mut mixer, &mut recorder);
    }

    run(context.as_mut(), &process_duration);

    {
        let mut recorder = recorder.borrow_mut();
        let recording =
            OwnedAudioBuffer::from_buffer(recorder.get_recording().expect("No recording was made"));
        write_buffer_into_file(recording, output_file);
    }
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

fn create_mixer(context: &dyn Context) -> Mixer {
    Mixer::mono_to_stereo_splitter(context.get_command_queue())
}

fn create_recorder(context: &mut dyn Context) -> Rc<RefCell<Recorder>> {
    let channel_count = 2;
    let sample_rate = context.get_sample_rate();
    Recorder::new(context, channel_count, sample_rate)
}

fn create_gain(context: &dyn Context) -> Gain {
    let gain_channel_count = 1;
    Gain::new(context.get_command_queue(), gain_channel_count)
}

fn make_connections(
    oscillators: &mut [Oscillator],
    gain: &mut Gain,
    mixer: &mut Mixer,
    recorder: &mut Recorder,
) {
    for oscillator in oscillators {
        oscillator.node.connect_to(&gain.node);
    }

    gain.node.connect_to(&mixer.node);

    mixer.node.connect_to(&recorder.node);
}

fn run(context: &mut dyn Context, end_time: &Timestamp) {
    context.start();

    while context.current_time() < *end_time {
        thread::sleep(Duration::from_secs_f64(1.0 / 60.0));
        context.process_notifications();
    }

    context.stop();

    thread::sleep(Duration::from_secs(1));
}
