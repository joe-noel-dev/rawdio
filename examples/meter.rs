use std::{cell::RefCell, rc::Rc, thread, time::Duration};

use rawdio::{
    create_engine, AudioBuffer, Context, Envelope, Gain, OwnedAudioBuffer, Sampler, Timestamp,
};
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
    let audio_callack = AudioCallback::new(audio_process, sample_rate);

    let channel_count = 2;
    let (mut sampler, duration) = create_sampler(context.as_mut(), sample);
    let gain = create_gain(context.as_mut(), channel_count, duration);
    let envelope = create_envelope(context.as_mut(), channel_count);

    sampler.node.connect_to(&gain.node);
    gain.node.connect_to(&envelope.borrow().node);
    gain.node.connect_to_output();

    sampler.start_now();

    context.start();

    while context.current_time() < Timestamp::from_duration(duration) {
        thread::sleep(Duration::from_millis(10));

        context.process_notifications();

        for event in envelope.borrow_mut().take_notifications() {
            println!("{}: {}", event.channel_index(), event.peak_level());
        }
    }

    context.stop();

    drop(audio_callack);
}

fn create_sampler(context: &mut dyn Context, sample: OwnedAudioBuffer) -> (Sampler, Duration) {
    let duration = sample.duration();

    (
        Sampler::new(context.get_command_queue(), sample.sample_rate(), sample),
        duration,
    )
}

fn create_gain(context: &mut dyn Context, channel_count: usize, sample_duration: Duration) -> Gain {
    let mut gain = Gain::new(context, channel_count);

    gain.gain.set_value_now(1.0);

    gain.gain
        .linear_ramp_to_value(0.0, Timestamp::from_duration(sample_duration));

    gain
}

fn create_envelope(context: &mut dyn Context, channel_count: usize) -> Rc<RefCell<Envelope>> {
    let attack_time = Duration::ZERO;
    let release_time = Duration::from_millis(300);
    let notification_frequency = 2.0;

    Envelope::new(
        context,
        channel_count,
        attack_time,
        release_time,
        notification_frequency,
    )
}
