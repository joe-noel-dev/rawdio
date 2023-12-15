use criterion::{criterion_group, criterion_main, Criterion};
use rawdio::{
    connect_nodes, create_engine_with_options, AudioProcess, EngineOptions, OwnedAudioBuffer, Pan,
    Timestamp,
};

struct Fixture {
    process: Box<dyn AudioProcess + Send>,
    pan: Pan,
    input_buffer: OwnedAudioBuffer,
    output_buffer: OwnedAudioBuffer,
}

impl Fixture {
    fn new() -> Self {
        let sample_rate = 48_000;
        let frame_count = 4_096;
        let channel_count = 2;

        let (mut context, process) =
            create_engine_with_options(EngineOptions::default().with_sample_rate(sample_rate));

        let mut pan = Pan::new(context.as_ref(), channel_count);

        connect_nodes!("input" => pan => "output");

        context.start();

        pan.pan().set_value_now(-0.5);

        let input_buffer = OwnedAudioBuffer::white_noise(frame_count, channel_count, sample_rate);
        let output_buffer = OwnedAudioBuffer::new(frame_count, channel_count, sample_rate);

        Self {
            process,
            pan,
            input_buffer,
            output_buffer,
        }
    }

    fn process(&mut self) {
        self.process
            .process(&self.input_buffer, &mut self.output_buffer);
    }
}

fn pan_benchmarks(c: &mut Criterion) {
    c.benchmark_group("Pan");

    c.bench_function("process fixed pan", |b| {
        let mut fixture = Fixture::new();

        b.iter(|| fixture.process());
    });

    c.bench_function("process pan with ramp", |b| {
        let mut fixture = Fixture::new();

        let start_time = Timestamp::zero();
        let end_time = Timestamp::from_samples(4_096.0, 48_000);

        fixture.pan.pan().set_value_now(-1.0);

        fixture
            .pan
            .pan()
            .linear_ramp_to_value(1.0, start_time, end_time);

        b.iter(|| fixture.process());
    });
}

criterion_group!(benches, pan_benchmarks);

criterion_main!(benches);
