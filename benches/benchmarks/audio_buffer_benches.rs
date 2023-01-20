use criterion::{black_box, criterion_group, Criterion};
use rawdio::{AudioBuffer, OwnedAudioBuffer, SampleLocation};

fn read_and_write_interleaved(source: &dyn AudioBuffer, destination: &mut dyn AudioBuffer) {
    for frame in 0..source.frame_count() {
        for channel in 0..source.channel_count() {
            let location = SampleLocation::new(channel, frame);
            let sample = source.get_sample(location);
            destination.set_sample(location, sample);
        }
    }
}

fn read_and_write_non_interleaved(source: &dyn AudioBuffer, destination: &mut dyn AudioBuffer) {
    for channel in 0..source.channel_count() {
        for frame in 0..source.frame_count() {
            let location = SampleLocation::new(channel, frame);
            let sample = source.get_sample(location);
            destination.set_sample(location, sample);
        }
    }
}

fn add_from(source: &dyn AudioBuffer, destination: &mut dyn AudioBuffer) {
    destination.add_from(
        source,
        SampleLocation::origin(),
        SampleLocation::origin(),
        destination.channel_count(),
        destination.frame_count(),
    );
}

fn audio_buffer_benchmarks(c: &mut Criterion) {
    c.benchmark_group("AudioBuffer");

    c.bench_function("read and write interleaved", |b| {
        let frame_count = 4096;
        let channel_count = 2;
        let sample_rate = 44_100;

        let source = OwnedAudioBuffer::white_noise(frame_count, channel_count, sample_rate);
        let mut destination = OwnedAudioBuffer::new(frame_count, channel_count, sample_rate);

        b.iter(|| read_and_write_interleaved(&source, &mut destination));

        black_box(destination);
    });

    c.bench_function("read and write non-interleaved", |b| {
        let frame_count = 4096;
        let channel_count = 2;
        let sample_rate = 44_100;

        let source = OwnedAudioBuffer::white_noise(frame_count, channel_count, sample_rate);
        let mut destination = OwnedAudioBuffer::new(frame_count, channel_count, sample_rate);

        b.iter(|| read_and_write_non_interleaved(&source, &mut destination));

        black_box(destination);
    });

    c.bench_function("add_from", |b| {
        let frame_count = 4096;
        let channel_count = 2;
        let sample_rate = 44_100;

        let source = OwnedAudioBuffer::white_noise(frame_count, channel_count, sample_rate);
        let mut destination =
            OwnedAudioBuffer::white_noise(frame_count, channel_count, sample_rate);

        b.iter(|| add_from(&source, &mut destination));

        black_box(destination);
    });
}

criterion_group!(benches, audio_buffer_benchmarks);
