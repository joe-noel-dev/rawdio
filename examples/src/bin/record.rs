use examples::{write_buffer_into_file, AudioCallback};
use rawdio::{
    create_engine_with_options, Context, EngineOptions, Level, Mixer, Recorder, Timestamp,
};
use std::{cell::RefCell, rc::Rc, thread, time::Duration};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Options {
    output_file: String,
    duration: f64,
}

fn main() {
    let options = Options::from_args();
    let output_file = &options.output_file;
    let record_duration = Timestamp::from_seconds(options.duration);

    let sample_rate = 44100;
    let channel_count = 2;

    let (mut context, process) =
        create_engine_with_options(EngineOptions::default().with_sample_rate(sample_rate));

    let audio_callback = AudioCallback::new(process, sample_rate);

    let recorder = create_recorder(context.as_mut(), channel_count, record_duration);

    let mixer = create_mixer(
        context.as_ref(),
        audio_callback.input_channel_count(),
        channel_count,
    );

    mixer.node.connect_to_input();
    mixer.node.connect_to(&recorder.borrow_mut().node);

    run(context.as_mut(), &record_duration);

    drop(audio_callback);

    finish_recording(recorder, output_file);
}

fn finish_recording(recorder: Rc<RefCell<Recorder>>, output_file: &str) {
    let mut recorder = recorder.borrow_mut();
    let recording = recorder.take_recording().expect("No recording was made");
    write_buffer_into_file(recording, output_file);
}

fn create_recorder(
    context: &mut dyn Context,
    channel_count: usize,
    end_time: Timestamp,
) -> Rc<RefCell<Recorder>> {
    let sample_rate = context.get_sample_rate();
    let recorder = Recorder::new(context, channel_count, sample_rate);

    recorder.borrow_mut().record_now();
    recorder.borrow_mut().stop_record_at_time(end_time);

    recorder
}

fn create_mixer(
    context: &dyn Context,
    input_channel_count: usize,
    output_channel_count: usize,
) -> Mixer {
    let mut mixer = Mixer::new(context, input_channel_count, output_channel_count);

    (0..output_channel_count).for_each(|output_channel| {
        (0..input_channel_count).for_each(|input_channel| {
            mixer.set_level(input_channel, output_channel, Level::unity());
        });
    });

    mixer
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
