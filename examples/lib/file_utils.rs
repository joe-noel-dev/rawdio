use rawdio::{
    AudioBuffer, AudioProcess, MutableBorrowedAudioBuffer, OwnedAudioBuffer, SampleLocation,
};

pub fn render_audio_process_to_file(
    sample_rate: usize,
    output_file: &str,
    audio_process: &mut dyn AudioProcess,
) {
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
        MutableBorrowedAudioBuffer::slice_frames(audio_buffer, frame_offset, frames_this_time);

    audio_process.process(&mut frame_buffer);

    for frame in 0..frame_buffer.frame_count() {
        for channel in 0..frame_buffer.channel_count() {
            let location = SampleLocation::new(channel, frame);
            let sample = frame_buffer.get_sample(location);
            let sample = sample.clamp(-1.0, 1.0);
            let sample = (sample * max_value as f32) as i32;
            writer.write_sample(sample).expect("Failed to write sample");
        }
    }
}
