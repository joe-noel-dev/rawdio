use criterion::{criterion_group, criterion_main, Criterion};
use rand::*;
use rust_audio_engine::{AudioBuffer, OwnedAudioBuffer, SampleLocation};

fn read_and_write_interleaved() {
    let mut buffer = OwnedAudioBuffer::new(1000000, 2, 44100);
    let mut rng = rand::thread_rng();

    for frame in 0..buffer.num_frames() {
        for channel in 0..buffer.num_channels() {
            buffer.set_sample(SampleLocation::new(channel, frame), rng.gen());
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
    let mut buffer = OwnedAudioBuffer::new(1000000, 2, 44100);
    let mut rng = rand::thread_rng();

    for channel in 0..buffer.num_channels() {
        for frame in 0..buffer.num_frames() {
            buffer.set_sample(SampleLocation::new(channel, frame), rng.gen());
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
