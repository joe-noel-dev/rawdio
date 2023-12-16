use criterion::{criterion_group, criterion_main, Criterion};
use rawdio::{prelude::*, Oscillator};

struct Fixture {
    audio_process: Box<dyn AudioProcess + Send>,
    context: Box<dyn Context>,
    input_buffer: OwnedAudioBuffer,
    output_buffer: OwnedAudioBuffer,
    _oscillator: Oscillator,
}

impl Fixture {
    pub fn new() -> Self {
        let sample_rate = 48_000;
        let (mut context, process) =
            create_engine_with_options(EngineOptions::default().with_sample_rate(sample_rate));

        let frame_count = 4096;
        let channel_count = 2;

        let frequency = 1_000.0;
        let oscillator = Oscillator::sine(context.as_ref(), frequency, channel_count);

        connect_nodes!(oscillator => "output");

        context.start();

        Self {
            audio_process: process,
            context,
            input_buffer: OwnedAudioBuffer::new(frame_count, channel_count, sample_rate),
            output_buffer: OwnedAudioBuffer::new(frame_count, channel_count, sample_rate),
            _oscillator: oscillator,
        }
    }

    fn process(&mut self) {
        self.audio_process
            .process(&self.input_buffer, &mut self.output_buffer);
        self.context.process_notifications();
    }
}

fn oscillator_benchmarks(c: &mut Criterion) {
    c.benchmark_group("Oscillator");

    c.bench_function("play oscillator", |b| {
        let mut fixture = Fixture::new();

        b.iter(|| fixture.process());
    });
}

criterion_group!(benches, oscillator_benchmarks);

criterion_main!(benches);
