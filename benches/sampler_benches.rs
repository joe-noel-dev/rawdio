use criterion::{criterion_group, criterion_main, Criterion};
use rawdio::{prelude::*, Sampler};

struct Fixture {
    audio_process: Box<dyn AudioProcess + Send>,
    context: Box<dyn Context>,
    input_buffer: OwnedAudioBuffer,
    output_buffer: OwnedAudioBuffer,
    _sampler: Sampler,
}

impl Fixture {
    pub fn new() -> Self {
        let sample_rate = 48_000;
        let (mut context, process) =
            create_engine_with_options(EngineOptions::default().with_sample_rate(sample_rate));

        let frame_count = 4096;
        let channel_count = 2;

        let sample = OwnedAudioBuffer::white_noise(frame_count, channel_count, sample_rate);

        let mut sampler = Sampler::new(context.as_ref(), sample);

        sampler.start_now();

        connect_nodes!(sampler => "output");

        context.start();

        Self {
            audio_process: process,
            context,
            input_buffer: OwnedAudioBuffer::new(frame_count, channel_count, sample_rate),
            output_buffer: OwnedAudioBuffer::new(frame_count, channel_count, sample_rate),
            _sampler: sampler,
        }
    }

    fn process(&mut self) {
        self.audio_process
            .process(&self.input_buffer, &mut self.output_buffer);
        self.context.process_notifications();
    }
}

fn sampler_benchmarks(c: &mut Criterion) {
    c.benchmark_group("Sampler");

    c.bench_function("play sample", |b| {
        let mut fixture = Fixture::new();

        b.iter(|| fixture.process());
    });
}

criterion_group!(benches, sampler_benchmarks);

criterion_main!(benches);
