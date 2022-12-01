use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rust_audio_engine::{AudioBuffer, OwnedAudioBuffer, SampleLocation};

fn read_and_write_interleaved() {
    let num_frames = 1024;
    let num_channels = 2;
    let sample_rate = 44_100;

    let mut buffer = OwnedAudioBuffer::new(num_frames, num_channels, sample_rate);

    for frame in 0..buffer.num_frames() {
        for channel in 0..buffer.num_channels() {
            buffer.set_sample(SampleLocation::new(channel, frame), 0.0);
        }
    }

    for frame in 0..buffer.num_frames() {
        for channel in 0..buffer.num_channels() {
            black_box(buffer.get_sample(SampleLocation::new(channel, frame)));
        }
    }
}

fn read_and_write_non_interleaved() {
    let num_frames = 1024;
    let num_channels = 2;
    let sample_rate = 44_100;

    let mut buffer = OwnedAudioBuffer::new(num_frames, num_channels, sample_rate);

    for channel in 0..buffer.num_channels() {
        for frame in 0..buffer.num_frames() {
            buffer.set_sample(SampleLocation::new(channel, frame), 0.0);
        }
    }

    for channel in 0..buffer.num_channels() {
        for frame in 0..buffer.num_frames() {
            black_box(buffer.get_sample(SampleLocation::new(channel, frame)));
        }
    }
}

fn add_from() {
    let num_frames = 50_000;
    let num_channels = 2;
    let sample_rate = 44_100;

    let mut buffer1 = OwnedAudioBuffer::new(num_frames, num_channels, sample_rate);
    let buffer2 = OwnedAudioBuffer::new(num_frames, num_channels, sample_rate);

    buffer1.add_from(
        &buffer2,
        SampleLocation::origin(),
        SampleLocation::origin(),
        num_channels,
        num_frames,
    );

    black_box(buffer1);
    black_box(buffer2);
}

fn benchmark(c: &mut Criterion) {
    c.bench_function("read and write interleaved", |b| {
        b.iter(read_and_write_interleaved)
    });

    c.bench_function("read and write non-interleaved", |b| {
        b.iter(read_and_write_non_interleaved)
    });

    c.bench_function("add_from", |b| b.iter(add_from));
}

criterion_group!(benches, benchmark);

criterion_main!(benches);
