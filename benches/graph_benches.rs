use criterion::{criterion_group, criterion_main, Criterion};
use rawdio::{create_engine, AudioProcess, Context, Gain, OwnedAudioBuffer, Sampler};

struct Fixture {
    input_buffer: OwnedAudioBuffer,
    output_buffer: OwnedAudioBuffer,
    _samplers: Vec<Sampler>,
    _gain_layers: Vec<Vec<Gain>>,
    _final_gain: Gain,
    audio_process: Box<dyn AudioProcess + Send>,
    context: Box<dyn Context>,
}

impl Fixture {
    pub fn new(layer_count: usize, nodes_per_layer: usize) -> Self {
        assert!(layer_count > 0);
        assert!(nodes_per_layer > 0);

        let sample_rate = 48_000;
        let (mut context, process) = create_engine(sample_rate);

        let frame_count = 4096;
        let channel_count = 2;

        let mut samplers = Vec::new();
        let mut gain_layers = Vec::new();

        let final_gain = Gain::new(context.as_ref(), channel_count);

        (0..layer_count).for_each(|layer| {
            let mut gains = Vec::new();

            match layer {
                0 => {
                    (0..nodes_per_layer).for_each(|_| {
                        let sample =
                            OwnedAudioBuffer::white_noise(frame_count, channel_count, sample_rate);
                        let mut sampler = Sampler::new(context.as_ref(), sample);

                        sampler.start_now();

                        samplers.push(sampler);
                    });
                }
                _ => {
                    (0..nodes_per_layer).for_each(|_| {
                        let mut gain = Gain::new(context.as_ref(), channel_count);

                        let value = 1.0 / (nodes_per_layer * layer_count) as f64;
                        gain.gain.set_value_now(value);

                        gains.push(gain);
                    });
                }
            }

            gain_layers.push(gains);
        });

        samplers.iter().enumerate().for_each(|(index, sampler)| {
            let layer = index / nodes_per_layer;

            if layer < layer_count {
                let gain_layer = &mut gain_layers[layer];
                gain_layer.iter().for_each(|gain| {
                    sampler.node.connect_to(&gain.node);
                });
            } else {
                sampler.node.connect_to(&final_gain.node);
            }
        });

        gain_layers
            .iter()
            .enumerate()
            .for_each(|(layer_index, gain_layer)| {
                let next_layer = layer_index + 1;

                gain_layer.iter().for_each(|gain_1| {
                    if next_layer < layer_count {
                        let gain_layer = &gain_layers[next_layer];
                        gain_layer.iter().for_each(|gain_2| {
                            gain_1.node.connect_to(&gain_2.node);
                        });
                    } else {
                        gain_1.node.connect_to(&final_gain.node);
                    }
                });
            });

        context.start();

        Self {
            audio_process: process,
            context,
            input_buffer: OwnedAudioBuffer::new(frame_count, channel_count, sample_rate),
            output_buffer: OwnedAudioBuffer::new(frame_count, channel_count, sample_rate),
            _samplers: samplers,
            _gain_layers: gain_layers,
            _final_gain: final_gain,
        }
    }

    fn process(&mut self) {
        self.audio_process
            .process(&self.input_buffer, &mut self.output_buffer);
        self.context.process_notifications();
    }
}

fn sampler_benchmarks(c: &mut Criterion) {
    c.benchmark_group("Graph");

    c.bench_function("single node graph", |b| {
        let layer_count = 1;
        let nodes_per_layer = 1;
        let mut fixture = Fixture::new(layer_count, nodes_per_layer);

        b.iter(|| fixture.process());
    });

    c.bench_function("deep graph", |b| {
        let layer_count = 2;
        let nodes_per_layer = 64;
        let mut fixture = Fixture::new(layer_count, nodes_per_layer);

        b.iter(|| fixture.process());
    });

    c.bench_function("wide graph", |b| {
        let layer_count = 64;
        let nodes_per_layer = 2;
        let mut fixture = Fixture::new(layer_count, nodes_per_layer);

        b.iter(|| fixture.process());
    });

    c.bench_function("varied graph", |b| {
        let layer_count = 12;
        let nodes_per_layer = 12;
        let mut fixture = Fixture::new(layer_count, nodes_per_layer);

        b.iter(|| fixture.process());
    });
}

criterion_group!(benches, sampler_benchmarks);

criterion_main!(benches);
