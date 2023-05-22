use criterion::{criterion_group, criterion_main, Criterion};
use rawdio::{
    create_engine_with_options, AudioProcess, Biquad, BiquadFilterType, EngineOptions,
    OwnedAudioBuffer, Timestamp,
};

struct Fixture {
    audio_process: Box<dyn AudioProcess + Send>,
    input_buffer: OwnedAudioBuffer,
    output_buffer: OwnedAudioBuffer,
    biquad: Biquad,
}

impl Fixture {
    pub fn new(filter_type: BiquadFilterType, cutoff_frequency: f64) -> Self {
        let sample_rate = 48_000;
        let (mut context, process) =
            create_engine_with_options(EngineOptions::default().with_sample_rate(sample_rate));

        let frame_count = 4096;
        let channel_count = 2;

        let sample = OwnedAudioBuffer::white_noise(frame_count, channel_count, sample_rate);

        let mut biquad = Biquad::new(context.as_ref(), channel_count, filter_type);

        biquad.frequency.set_value_now(cutoff_frequency);

        biquad.node.connect_to_input();
        biquad.node.connect_to_output();

        context.start();

        Self {
            audio_process: process,
            input_buffer: sample,
            output_buffer: OwnedAudioBuffer::new(frame_count, channel_count, sample_rate),
            biquad,
        }
    }

    fn ramp_to_frequency_at_time(
        &mut self,
        value: f64,
        start_time: Timestamp,
        end_time: Timestamp,
    ) {
        self.biquad
            .frequency
            .linear_ramp_to_value(value, start_time, end_time);
    }

    fn process(&mut self) {
        self.audio_process
            .process(&self.input_buffer, &mut self.output_buffer);
    }
}

fn biquad_benchmarks(c: &mut Criterion) {
    c.benchmark_group("Biquad Filter");

    c.bench_function("process low pass", |b| {
        let cutoff = 1_000.0;
        let mut fixture = Fixture::new(BiquadFilterType::LowPass, cutoff);

        b.iter(|| fixture.process());
    });

    c.bench_function("process low pass with frequency ramp", |b| {
        let cutoff = 1_000.0;
        let mut fixture = Fixture::new(BiquadFilterType::LowPass, cutoff);

        fixture.ramp_to_frequency_at_time(2_000.0, Timestamp::zero(), Timestamp::from_seconds(1.0));

        b.iter(|| fixture.process());
    });
}

criterion_group!(benches, biquad_benchmarks);

criterion_main!(benches);
