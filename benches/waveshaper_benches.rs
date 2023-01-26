use criterion::{criterion_group, criterion_main, Criterion};
use rawdio::{create_engine, AudioProcess, OwnedAudioBuffer, Waveshaper};

struct Fixture {
    audio_process: Box<dyn AudioProcess + Send>,
    input_buffer: OwnedAudioBuffer,
    output_buffer: OwnedAudioBuffer,
    _waveshaper: Waveshaper,
}

impl Fixture {
    pub fn new() -> Self {
        let sample_rate = 48_000;
        let (mut context, process) = create_engine(sample_rate);

        let frame_count = 4096;
        let channel_count = 2;

        let sample = OwnedAudioBuffer::white_noise(frame_count, channel_count, sample_rate);

        let waveshaper = Waveshaper::tanh(context.as_ref(), channel_count);

        waveshaper.node.connect_to_input();
        waveshaper.node.connect_to_output();

        context.start();

        Self {
            audio_process: process,
            input_buffer: sample,
            output_buffer: OwnedAudioBuffer::new(frame_count, channel_count, sample_rate),
            _waveshaper: waveshaper,
        }
    }

    fn process(&mut self) {
        self.audio_process
            .process(&self.input_buffer, &mut self.output_buffer);
    }
}

fn biquad_benchmarks(c: &mut Criterion) {
    c.benchmark_group("Waveshaper");

    c.bench_function("process waveshaper", |b| {
        let mut fixture = Fixture::new();

        b.iter(|| fixture.process());
    });
}

criterion_group!(benches, biquad_benchmarks);

criterion_main!(benches);
