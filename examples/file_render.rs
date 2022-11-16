use rust_audio_engine::{
    create_context, AudioBuffer, AudioProcess, BorrowedAudioBuffer, Context, Gain, Oscillator,
    OwnedAudioBuffer, Pan, Timestamp,
};
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
    let mut context = create_context(sample_rate);
    let mut audio_process = context.get_audio_process();

    let mut oscillators = create_oscillators(context.as_ref());
    let mut gain = create_gain(context.as_ref());
    let mut pan = create_pan(context.as_ref());

    make_connections(&mut oscillators, &mut gain, &mut pan);

    context.start();
    render_to_file(sample_rate, output_file, audio_process.as_mut());
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

fn render_to_file(sample_rate: usize, output_file: &str, audio_process: &mut dyn AudioProcess) {
    let bits_per_sample = 24;
    let max_value = 2_i32.pow(bits_per_sample - 1) - 1;
    let num_channels = 2;

    let mut writer = create_writer(num_channels, sample_rate, bits_per_sample, output_file);

    let length_in_seconds = 4.0;
    let total_frame_count = sample_rate * length_in_seconds as usize;

    let max_frame_count = 1024;
    let mut audio_buffer = OwnedAudioBuffer::new(total_frame_count, num_channels, sample_rate);

    let mut position = 0;
    while position < total_frame_count {
        let frames_this_time = std::cmp::min(max_frame_count, total_frame_count - position);
        process_block(
            frames_this_time,
            position,
            &mut audio_buffer,
            audio_process,
            max_value,
            &mut writer,
        );
        position += frames_this_time;
    }
}

fn create_writer(
    num_channels: usize,
    sample_rate: usize,
    bits_per_sample: u32,
    output_file: &str,
) -> hound::WavWriter<std::io::BufWriter<std::fs::File>> {
    let file_spec = hound::WavSpec {
        channels: num_channels as u16,
        sample_rate: sample_rate as u32,
        bits_per_sample: bits_per_sample as u16,
        sample_format: hound::SampleFormat::Int,
    };

    hound::WavWriter::create(output_file, file_spec).expect("Unable to create file writer")
}

fn process_block<T>(
    frames_this_time: usize,
    frame_offset: usize,
    audio_buffer: &mut dyn AudioBuffer,
    audio_process: &mut dyn AudioProcess,
    max_value: i32,
    writer: &mut hound::WavWriter<T>,
) where
    T: std::io::Write + std::io::Seek,
{
    let mut frame_buffer =
        BorrowedAudioBuffer::slice_frames(audio_buffer, frame_offset, frames_this_time);

    audio_process.process(&mut frame_buffer);

    for location in frame_buffer.frame_iter() {
        let sample = frame_buffer.get_sample(location);
        let sample = sample.clamp(-1.0, 1.0);
        let sample = (sample * max_value as f32) as i32;
        writer.write_sample(sample).expect("Failed to write sample");
    }
}
