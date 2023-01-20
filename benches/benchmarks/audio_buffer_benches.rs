use criterion::{black_box, criterion_group, Criterion};
use rawdio::{AudioBuffer, OwnedAudioBuffer, SampleLocation};

fn read_interleaved(destination: &mut dyn AudioBuffer, interleaved_buffer: &[f32]) {
    destination.fill_from_interleaved(
        interleaved_buffer,
        destination.channel_count(),
        destination.frame_count(),
    );
}

fn write_interleaved(source: &dyn AudioBuffer, interleaved_buffer: &mut [f32]) {
    source.copy_to_interleaved(
        interleaved_buffer,
        source.channel_count(),
        source.frame_count(),
    );
}

fn read_and_write_non_interleaved(source: &dyn AudioBuffer, destination: &mut dyn AudioBuffer) {
    destination.copy_from(
        source,
        SampleLocation::origin(),
        SampleLocation::origin(),
        destination.channel_count(),
        destination.frame_count(),
    );
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

    c.bench_function("read interleaved", |b| {
        const FRAME_COUNT: usize = 4096;
        const CHANNEL_COUNT: usize = 2;
        const SAMPLE_RATE: usize = 44_100;

        let mut interleaved_buffer: Vec<f32> = Vec::from([0.0_f32; FRAME_COUNT * CHANNEL_COUNT]);
        let source = OwnedAudioBuffer::white_noise(FRAME_COUNT, CHANNEL_COUNT, SAMPLE_RATE);
        source.copy_to_interleaved(&mut interleaved_buffer, CHANNEL_COUNT, FRAME_COUNT);

        let mut destination = OwnedAudioBuffer::new(FRAME_COUNT, CHANNEL_COUNT, SAMPLE_RATE);

        b.iter(|| read_interleaved(&mut destination, &interleaved_buffer));

        black_box(destination);
    });

    c.bench_function("write interleaved", |b| {
        const FRAME_COUNT: usize = 4096;
        const CHANNEL_COUNT: usize = 2;
        const SAMPLE_RATE: usize = 44_100;

        let mut interleaved_buffer: Vec<f32> = Vec::from([0.0_f32; FRAME_COUNT * CHANNEL_COUNT]);

        let source = OwnedAudioBuffer::white_noise(FRAME_COUNT, CHANNEL_COUNT, SAMPLE_RATE);

        b.iter(|| write_interleaved(&source, &mut interleaved_buffer));

        black_box(interleaved_buffer);
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
