use std::{cell::RefCell, rc::Rc, time::Duration};

use criterion::{criterion_group, criterion_main, Criterion};
use rawdio::{create_engine, AudioProcess, Context, Envelope, OwnedAudioBuffer};

struct Fixture {
    audio_process: Box<dyn AudioProcess + Send>,
    context: Box<dyn Context>,
    input_buffer: OwnedAudioBuffer,
    output_buffer: OwnedAudioBuffer,
    _envelope: Rc<RefCell<Envelope>>,
}

impl Fixture {
    pub fn new(attack_time: Duration, release_time: Duration, notification_frequency: f64) -> Self {
        let sample_rate = 48_000;
        let (mut context, process) = create_engine(sample_rate);

        let frame_count = 4096;
        let channel_count = 2;

        let sample = OwnedAudioBuffer::white_noise(frame_count, channel_count, sample_rate);

        let envelope = Envelope::new(
            context.as_mut(),
            channel_count,
            attack_time,
            release_time,
            notification_frequency,
        );

        envelope.borrow_mut().node.connect_to_input();

        context.start();

        Self {
            audio_process: process,
            context,
            input_buffer: sample,
            output_buffer: OwnedAudioBuffer::new(frame_count, channel_count, sample_rate),
            _envelope: envelope,
        }
    }

    fn process(&mut self) {
        self.audio_process
            .process(&self.input_buffer, &mut self.output_buffer);
        self.context.process_notifications();
    }
}

fn envelope_benchmarks(c: &mut Criterion) {
    c.benchmark_group("Envelope");

    c.bench_function("process envelope", |b| {
        let attack_time = Duration::from_millis(10);
        let release_time = Duration::from_millis(100);
        let notification_frequency = 60.0;

        let mut fixture = Fixture::new(attack_time, release_time, notification_frequency);

        b.iter(|| fixture.process());
    });
}

criterion_group!(benches, envelope_benchmarks);

criterion_main!(benches);
