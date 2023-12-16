use criterion::{criterion_group, criterion_main, Criterion};
use rawdio::{prelude::*, Compressor};

struct Fixture {
    audio_process: Box<dyn AudioProcess + Send>,
    input_buffer: OwnedAudioBuffer,
    output_buffer: OwnedAudioBuffer,
    compressor: Compressor,
}

impl Fixture {
    pub fn new() -> Self {
        let sample_rate = 48_000;
        let (mut context, process) =
            create_engine_with_options(EngineOptions::default().with_sample_rate(sample_rate));

        let frame_count = 4096;
        let channel_count = 2;

        let sample = OwnedAudioBuffer::white_noise(frame_count, channel_count, sample_rate);

        let compressor = Compressor::new(context.as_ref(), channel_count);

        connect_nodes!("input" => compressor => "output");

        context.start();

        Self {
            audio_process: process,
            input_buffer: sample,
            output_buffer: OwnedAudioBuffer::new(frame_count, channel_count, sample_rate),
            compressor,
        }
    }

    fn process(&mut self) {
        self.audio_process
            .process(&self.input_buffer, &mut self.output_buffer);
    }
}

fn compressor_benchmarks(c: &mut Criterion) {
    c.benchmark_group("Compressor");

    c.bench_function("process compressor", |b| {
        let mut fixture = Fixture::new();

        fixture.compressor.attack().set_value_now(1.0);
        fixture.compressor.release().set_value_now(10.0);
        fixture.compressor.dry().set_value_now(0.5);
        fixture.compressor.wet().set_value_now(0.5);
        fixture.compressor.threshold().set_value_now(-12.0);
        fixture.compressor.knee().set_value_now(6.0);

        b.iter(|| fixture.process());
    });
}

criterion_group!(benches, compressor_benchmarks);

criterion_main!(benches);
