use criterion::criterion_main;

mod benchmarks;

criterion_main! {
    benchmarks::adsr_benches::benches,
    benchmarks::audio_buffer_benches::benches,
    benchmarks::biquad_benches::benches,
    benchmarks::envelope_benches::benches,
    benchmarks::gain_benches::benches,
}
