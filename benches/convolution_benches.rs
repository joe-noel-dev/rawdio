use std::time::Duration;

use criterion::{criterion_group, criterion_main, Criterion};
use rawdio::{
    create_engine_with_options, AudioProcess, Convolution, EngineOptions, OwnedAudioBuffer,
};

#[allow(dead_code)]
struct Fixture {
    audio_process: Box<dyn AudioProcess + Send>,
    input_buffer: OwnedAudioBuffer,
    output_buffer: OwnedAudioBuffer,
    convolution: Convolution,
}

impl Fixture {
    pub fn new(impulse_duration: Duration) -> Self {
        let sample_rate = 48_000;

        let (mut context, process) =
            create_engine_with_options(EngineOptions::default().with_sample_rate(sample_rate));

        let frame_count = 4_096;
        let channel_count = 2;

        let sample = OwnedAudioBuffer::white_noise(frame_count, channel_count, sample_rate);

        let impulse_sample_count =
            (impulse_duration.as_secs_f64() * sample_rate as f64).floor() as usize;

        let impulse =
            OwnedAudioBuffer::white_noise(impulse_sample_count, channel_count, sample_rate);

        let convolution = Convolution::new(context.as_ref(), channel_count, impulse);

        convolution.node.connect_to_input();
        convolution.node.connect_to_output();

        context.start();

        Self {
            audio_process: process,
            input_buffer: sample,
            output_buffer: OwnedAudioBuffer::new(frame_count, channel_count, sample_rate),
            convolution,
        }
    }

    fn process(&mut self) {
        self.audio_process
            .process(&self.input_buffer, &mut self.output_buffer);
    }
}

fn convolution_benches(c: &mut Criterion) {
    c.benchmark_group("Convolution");

    c.bench_function("process 2 second convolution", |b| {
        let impulse_duration = Duration::from_secs(2);
        let mut fixture = Fixture::new(impulse_duration);
        b.iter(|| fixture.process());
    });
}

criterion_group!(benches, convolution_benches);

criterion_main!(benches);
