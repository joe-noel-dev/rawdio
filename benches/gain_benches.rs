use criterion::{criterion_group, criterion_main, Criterion};
use rawdio::{
    connect_nodes, create_engine_with_options, AudioProcess, EngineOptions, Gain, OwnedAudioBuffer,
    Timestamp,
};

struct Fixture {
    process: Box<dyn AudioProcess + Send>,
    gain: Gain,
    input_buffer: OwnedAudioBuffer,
    output_buffer: OwnedAudioBuffer,
}

impl Fixture {
    fn new() -> Self {
        let sample_rate = 48_000;

        let (mut context, process) =
            create_engine_with_options(EngineOptions::default().with_sample_rate(sample_rate));

        let channel_count = 2;

        let mut gain = Gain::new(context.as_ref(), channel_count);

        gain.gain.set_value_at_time(1.0, Timestamp::zero());

        connect_nodes!("input" => gain => "output");

        context.start();

        let frame_count = 4096;

        let input_buffer = OwnedAudioBuffer::white_noise(frame_count, channel_count, sample_rate);
        let output_buffer = OwnedAudioBuffer::new(frame_count, channel_count, sample_rate);

        Self {
            process,
            gain,
            input_buffer,
            output_buffer,
        }
    }

    fn process(&mut self) {
        self.process
            .process(&self.input_buffer, &mut self.output_buffer);
    }
}

fn gain_benchmarks(c: &mut Criterion) {
    c.benchmark_group("Gain");

    c.bench_function("process fixed gain", |b| {
        let mut fixture = Fixture::new();

        b.iter(|| fixture.process());
    });

    c.bench_function("process gain with ramp", |b| {
        let mut fixture = Fixture::new();

        let start_time = Timestamp::zero();
        let end_time = Timestamp::from_seconds(2.0);

        fixture
            .gain
            .gain
            .linear_ramp_to_value(0.0, start_time, end_time);

        b.iter(|| fixture.process());
    });
}

criterion_group!(benches, gain_benchmarks);

criterion_main!(benches);
