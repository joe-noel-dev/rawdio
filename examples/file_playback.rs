use std::{thread, time};

use rust_audio_engine::{
    AudioBuffer, Context, Gain, Level, Node, OwnedAudioBuffer, SampleLocation, Sampler, Timestamp,
};
use structopt::StructOpt;

use crate::audio_callback::AudioCallback;

#[path = "./lib/audio_callback.rs"]
mod audio_callback;

#[derive(Debug, StructOpt)]
struct Options {
    file_to_play: String,
}

fn main() {
    let options = Options::from_args();
    play_file(&options.file_to_play);
}

fn play_file(file_to_play: &str) {
    let (sample, sample_rate) = read_file_into_buffer(file_to_play);

    let mut context = Context::new(sample_rate);
    let _audio_callack = AudioCallback::new(context.get_audio_process(), sample_rate);

    let length_in_seconds = sample.length_in_seconds().ceil() as u64;
    let length_in_samples = sample.num_frames();
    let mut sampler = Sampler::new(context.get_command_queue(), sample_rate, sample);
    let mut gain = Gain::new(context.get_command_queue());

    sampler.connect_to(gain.get_id());
    sampler.start_now();
    sampler.enable_loop(
        Timestamp::zero(),
        Timestamp::from_samples(length_in_samples as f64, sample_rate),
    );

    gain.connect_to_output();

    gain.gain
        .set_value_at_time(Level::from_db(-6.0).as_gain(), Timestamp::zero());

    context.start();

    thread::sleep(time::Duration::from_secs(4 * length_in_seconds));

    context.process_notifications();
    context.stop();

    thread::sleep(time::Duration::from_secs(1));
}

fn read_file_into_buffer(file_path: &str) -> (OwnedAudioBuffer, usize) {
    let mut reader = hound::WavReader::open(file_path).unwrap();
    let file_specification = reader.spec();
    let num_channels = file_specification.channels as usize;
    let sample_rate = file_specification.sample_rate as usize;
    let num_samples = reader.len() as usize;
    let num_frames = num_samples / num_channels;

    let mut output_buffer = OwnedAudioBuffer::new(num_frames, num_channels, sample_rate);

    let max_value = 2_i32.pow(file_specification.bits_per_sample as u32 - 1) - 1;

    for (position, sample) in reader.samples::<i32>().enumerate() {
        if let Ok(sample) = sample {
            for channel in 0..file_specification.channels {
                let frame = position / num_channels;
                let sample = (sample as f64 / max_value as f64) as f32;
                output_buffer.set_sample(SampleLocation::new(channel as usize, frame), sample);
            }
        }
    }

    (output_buffer, sample_rate)
}
