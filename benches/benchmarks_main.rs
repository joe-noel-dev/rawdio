use criterion::criterion_main;

mod benchmarks;

criterion_main! {
    benchmarks::audio_buffer_benches::benches,
    benchmarks::gain_benches::benches,
    benchmarks::biquad_benches::benches,
    benchmarks::adsr_benches::benches,
}
