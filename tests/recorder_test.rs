use std::time::Duration;

use itertools::izip;
use rawdio::{
    create_engine, AudioBuffer, OwnedAudioBuffer, Recorder, SampleLocation, Sampler, Timestamp,
};

use rand::Rng;

fn create_white_noise_buffer(
    frame_count: usize,
    sample_rate: usize,
    channel_count: usize,
) -> OwnedAudioBuffer {
    let mut buffer = OwnedAudioBuffer::new(frame_count, channel_count, sample_rate);

    let mut random_generator = rand::thread_rng();

    for frame in buffer.frame_iter() {
        let sample_value = random_generator.gen_range(-1.0..=1.0);
        buffer.set_sample(frame, sample_value);
    }

    buffer
}

fn record_sampler(
    sample_rate: usize,
    channel_count: usize,
    buffer: &OwnedAudioBuffer,
) -> OwnedAudioBuffer {
    let (mut context, mut process) = create_engine(sample_rate);

    let mut sampler = Sampler::new(context.get_command_queue(), sample_rate, buffer.clone());
    let recorder = Recorder::new(context.as_mut(), channel_count, sample_rate);

    sampler.node.connect_to(&recorder.borrow_mut().node);

    sampler.start_now();
    recorder.borrow_mut().record_now();
    recorder
        .borrow_mut()
        .stop_record_at_time(Timestamp::from_samples(
            (buffer.frame_count() + 1) as f64,
            sample_rate,
        ));

    let mut output_buffer =
        OwnedAudioBuffer::new(2 * buffer.frame_count(), channel_count, sample_rate);

    context.start();
    process.process(&mut output_buffer);
    sampler.stop_now();

    context.process_notifications();

    context.stop();

    let recording = recorder
        .borrow_mut()
        .take_recording()
        .expect("No recording was made");

    recording
}

fn has_identical_data(buffer_1: &dyn AudioBuffer, buffer_2: &dyn AudioBuffer) -> bool {
    if buffer_1.channel_count() != buffer_2.channel_count() {
        return false;
    }

    if buffer_1.frame_count() != buffer_2.frame_count() {
        return false;
    }

    for channel in 0..buffer_1.channel_count() {
        let location = SampleLocation::channel(channel);
        let buffer_1_data = buffer_1.get_channel_data(location);
        let buffer_2_data = buffer_2.get_channel_data(location);

        if izip!(buffer_1_data.iter(), buffer_2_data.iter())
            .any(|(buffer_1_sample, buffer_2_sample)| buffer_1_sample != buffer_2_sample)
        {
            return false;
        }
    }

    true
}

#[test]
fn records_data() {
    let sample_duration = Duration::from_secs(1);
    let sample_rate = 48_000;

    let sample_duration = Timestamp::from_duration(sample_duration);
    let sample_frame_count = sample_duration.as_samples(sample_rate).ceil() as usize;

    let channel_count = 2;

    let buffer = create_white_noise_buffer(sample_frame_count, sample_rate, channel_count);
    let recorded_buffer = record_sampler(sample_rate, channel_count, &buffer);

    for channel in 0..channel_count {
        assert!(!recorded_buffer.channel_is_silent(channel));
    }

    assert!(has_identical_data(&buffer, &recorded_buffer));
}
