use rawdio::{
    AudioBuffer, AudioProcess, MutableBorrowedAudioBuffer, OwnedAudioBuffer, SampleLocation,
};

pub fn read_file_into_buffer(file_path: &str) -> OwnedAudioBuffer {
    let mut reader = hound::WavReader::open(file_path).expect("Unable to open file for reading");
    let file_specification = reader.spec();
    let num_channels = file_specification.channels as usize;
    let sample_rate = file_specification.sample_rate as usize;
    let num_samples = reader.len() as usize;
    let num_frames = num_samples / num_channels;

    let mut output_buffer = OwnedAudioBuffer::new(num_frames, num_channels, sample_rate);

    let max_value = 2_i32.pow(file_specification.bits_per_sample as u32 - 1) - 1;

    for (position, sample) in reader.samples::<i32>().enumerate() {
        if let Ok(sample) = sample {
            let frame = position / num_channels;
            let channel = position % num_channels;
            let sample_location = SampleLocation::new(channel as usize, frame);
            let sample = (sample as f64 / max_value as f64) as f32;
            output_buffer.set_sample(sample_location, sample);
        }
    }

    output_buffer
}

pub fn write_buffer_into_file(buffer: OwnedAudioBuffer, output_file: &str) {
    let bits_per_sample = 24;
    let max_value = 2_i32.pow(bits_per_sample - 1) - 1;

    let mut writer = create_writer(
        buffer.channel_count(),
        buffer.sample_rate(),
        bits_per_sample,
        output_file,
    );

    for frame in 0..buffer.frame_count() {
        for channel in 0..buffer.channel_count() {
            let location = SampleLocation::new(channel, frame);
            let sample = buffer.get_sample(location);
            let sample = sample.clamp(-1.0, 1.0);
            let sample = (sample * max_value as f32) as i32;
            writer.write_sample(sample).expect("Failed to write sample");
        }
    }
}

pub fn render_audio_process_to_file(
    sample_rate: usize,
    output_file: &str,
    mut audio_process: Box<dyn AudioProcess>,
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
            audio_process.as_mut(),
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
    let input_buffer = OwnedAudioBuffer::new(
        frames_this_time,
        audio_buffer.channel_count(),
        audio_buffer.sample_rate(),
    );

    let mut frame_buffer =
        MutableBorrowedAudioBuffer::slice_frames(audio_buffer, frame_offset, frames_this_time);

    audio_process.process(&input_buffer, &mut frame_buffer);

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
