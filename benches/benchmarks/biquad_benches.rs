use criterion::{criterion_group, Criterion};
use rawdio::{
    create_engine, AudioProcess, Biquad, BiquadFilterType, Context, OwnedAudioBuffer, Sampler,
    Timestamp,
};

struct Fixture {
    audio_process: Box<dyn AudioProcess + Send>,
    _context: Box<dyn Context>,
    output_buffer: OwnedAudioBuffer,
    biquad: Biquad,
    _sampler: Sampler,
}

impl Fixture {
    pub fn new(filter_type: BiquadFilterType, cutoff_frequency: f64) -> Self {
        let sample_rate = 48_000;
        let (mut context, process) = create_engine(sample_rate);

        let frame_count = 4096;
        let channel_count = 2;

        let sample = OwnedAudioBuffer::white_noise(frame_count, channel_count, sample_rate);

        let mut sampler = Sampler::new(context.get_command_queue(), sample_rate, sample);
        sampler.start_now();

        let mut biquad = Biquad::new(context.as_ref(), channel_count, filter_type);

        biquad.frequency.set_value_now(cutoff_frequency);

        sampler.node.connect_to(&biquad.node);
        biquad.node.connect_to_output();

        context.start();

        Self {
            audio_process: process,
            _context: context,
            output_buffer: OwnedAudioBuffer::new(frame_count, channel_count, sample_rate),
            biquad,
            _sampler: sampler,
        }
    }

    fn ramp_to_frequency_at_time(&mut self, value: f64, end_time: Timestamp) {
        self.biquad.frequency.linear_ramp_to_value(value, end_time);
    }

    fn process(&mut self) {
        self.audio_process.process(&mut self.output_buffer)
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

        fixture.ramp_to_frequency_at_time(2_000.0, Timestamp::from_seconds(1.0));

        b.iter(|| fixture.process());
    });
}

criterion_group!(benches, biquad_benchmarks);
