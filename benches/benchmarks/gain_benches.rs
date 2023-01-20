use criterion::{black_box, criterion_group, Criterion};
use rawdio::{create_engine, Gain, OwnedAudioBuffer, Timestamp};

fn gain_benchmarks(c: &mut Criterion) {
    c.benchmark_group("Gain");

    c.bench_function("process gain", |b| {
        let sample_rate = 48_000;
        let (mut context, mut process) = create_engine(sample_rate);
        let channel_count = 1;
        let mut gain = Gain::new(context.get_command_queue(), channel_count);

        gain.gain.set_value_at_time(1.0, Timestamp::zero());

        gain.gain
            .linear_ramp_to_value(0.0, Timestamp::from_seconds(2.0));

        gain.node.connect_to_output();

        context.start();

        let frame_count = 4096;
        let mut output_buffer = OwnedAudioBuffer::new(frame_count, channel_count, sample_rate);

        b.iter(|| process.process(&mut output_buffer));

        black_box(output_buffer);
    });
}

criterion_group!(benches, gain_benchmarks);
