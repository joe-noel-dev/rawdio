use criterion::{criterion_group, Criterion};
use rawdio::{create_engine, AudioProcess, Gain, OwnedAudioBuffer, Timestamp};

struct Fixture {
    process: Box<dyn AudioProcess + Send>,
    gain: Gain,
    output_buffer: OwnedAudioBuffer,
}

impl Fixture {
    fn new() -> Self {
        let sample_rate = 48_000;

        let (mut context, process) = create_engine(sample_rate);

        let channel_count = 2;

        let mut gain = Gain::new(context.get_command_queue(), channel_count);

        gain.gain.set_value_at_time(1.0, Timestamp::zero());

        gain.node.connect_to_output();

        context.start();

        let frame_count = 4096;
        let output_buffer = OwnedAudioBuffer::new(frame_count, channel_count, sample_rate);

        Self {
            process,
            gain,
            output_buffer,
        }
    }

    fn process(&mut self) {
        self.process.process(&mut self.output_buffer);
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

        fixture
            .gain
            .gain
            .linear_ramp_to_value(0.0, Timestamp::from_seconds(2.0));

        b.iter(|| fixture.process());
    });
}

criterion_group!(benches, gain_benchmarks);
