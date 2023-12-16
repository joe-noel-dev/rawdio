use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rawdio::{prelude::*, Adsr};

fn adsr_benchmarks(c: &mut Criterion) {
    c.benchmark_group("ADSR");

    c.bench_function("process adsr", |bencher| {
        let sample_rate = 48_000;
        let (mut context, mut process) =
            create_engine_with_options(EngineOptions::default().with_sample_rate(sample_rate));

        let frame_count = 4_096;
        let channel_count = 2;

        let mut adsr = Adsr::new(context.as_ref(), channel_count, sample_rate);
        adsr.set_attack_time(Duration::from_millis(10));
        adsr.set_decay_time(Duration::from_millis(20));
        adsr.set_sustain_level(Level::from_db(-6.0));
        adsr.set_release_time(Duration::from_millis(40));

        adsr.note_on_at_time(Timestamp::zero());
        adsr.note_off_at_time(Timestamp::from_duration(Duration::from_millis(35)));

        connect_nodes!("input" => adsr => "output");

        context.start();

        let input_buffer = OwnedAudioBuffer::white_noise(frame_count, channel_count, sample_rate);
        let mut output_buffer = OwnedAudioBuffer::new(frame_count, channel_count, sample_rate);

        bencher.iter(|| process.process(&input_buffer, &mut output_buffer));

        black_box(output_buffer);
    });
}

criterion_group!(benches, adsr_benchmarks);

criterion_main!(benches);
