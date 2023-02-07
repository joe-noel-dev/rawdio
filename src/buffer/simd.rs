use std::iter::zip;

pub fn mix(source: &[f32], destination: &mut [f32]) {
    debug_assert_eq!(source.len(), destination.len());

    const VECTOR_SIZE: usize = 16;

    let (source_pre, source_main, source_post) = source.as_simd::<VECTOR_SIZE>();
    let (dest_pre, dest_main, dest_post) = destination.as_simd_mut::<VECTOR_SIZE>();

    if source_pre.len() == dest_pre.len() {
        for (input, output) in zip(source_pre, dest_pre) {
            *output += *input;
        }

        for (input, output) in zip(source_main, dest_main) {
            *output += *input;
        }

        for (input, output) in zip(source_post, dest_post) {
            *output += *input;
        }
    } else {
        for (input, output) in zip(source, destination) {
            *output = *input;
        }
    }
}

pub fn apply_gain(data: &mut [f32], gain: &[f32]) {
    debug_assert_eq!(data.len(), gain.len());

    const VECTOR_SIZE: usize = 16;

    let (gain_pre, gain_main, gain_post) = gain.as_simd::<VECTOR_SIZE>();
    let (data_pre, data_main, data_post) = data.as_simd_mut::<VECTOR_SIZE>();

    if gain_pre.len() == data_pre.len() {
        for (gain, sample) in zip(gain_pre, data_pre) {
            *sample *= *gain;
        }

        for (gain, sample) in zip(gain_main, data_main) {
            *sample *= *gain;
        }

        for (gain, sample) in zip(gain_post, data_post) {
            *sample *= *gain;
        }
    } else {
        for (gain, sample) in zip(gain, data) {
            *sample *= *gain;
        }
    }
}
