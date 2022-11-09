use criterion::{criterion_group, criterion_main, Criterion};
use rust_audio_engine::{AudioBuffer, OwnedAudioBuffer, SampleLocation};

fn read_and_write_interleaved() {
    let num_frames = 1_000_000;
    let num_channels = 2;
    let sample_rate = 44_100;

    let mut buffer = OwnedAudioBuffer::new(num_frames, num_channels, sample_rate);

    for frame in 0..buffer.num_frames() {
        for channel in 0..buffer.num_channels() {
            buffer.set_sample(SampleLocation::new(channel, frame), 0.0);
        }
    }

    let mut _total = 0.0;
    for frame in 0..buffer.num_frames() {
        for channel in 0..buffer.num_channels() {
            _total += buffer.get_sample(SampleLocation::new(channel, frame));
        }
    }
}

fn read_and_write_non_interleaved() {
    let num_frames = 1_000_000;
    let num_channels = 2;
    let sample_rate = 44_100;

    let mut buffer = OwnedAudioBuffer::new(num_frames, num_channels, sample_rate);

    for channel in 0..buffer.num_channels() {
        for frame in 0..buffer.num_frames() {
            buffer.set_sample(SampleLocation::new(channel, frame), 0.0);
        }
    }

    let mut _total = 0.0;
    for channel in 0..buffer.num_channels() {
        for frame in 0..buffer.num_frames() {
            _total += buffer.get_sample(SampleLocation::new(channel, frame));
        }
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("read and write interleaved", |b| {
        b.iter(read_and_write_interleaved)
    });

    c.bench_function("read and write non-interleaved", |b| {
        b.iter(read_and_write_non_interleaved)
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
