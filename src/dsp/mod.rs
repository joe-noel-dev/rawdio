#![allow(dead_code)]

use std::{iter::zip, simd::Simd};

use itertools::izip;

pub fn mix_into(source: &[f32], destination: &mut [f32]) {
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
            *output += *input;
        }
    }
}

pub fn mix_into_with_gain(source: &[f32], destination: &mut [f32], gain: f32) {
    debug_assert_eq!(source.len(), destination.len());

    const VECTOR_SIZE: usize = 16;

    let (source_pre, source_main, source_post) = source.as_simd::<VECTOR_SIZE>();
    let (dest_pre, dest_main, dest_post) = destination.as_simd_mut::<VECTOR_SIZE>();
    let gain_vector = Simd::<f32, VECTOR_SIZE>::from_array([gain; VECTOR_SIZE]);

    if source_pre.len() == dest_pre.len() {
        for (input, output) in zip(source_pre, dest_pre) {
            *output += *input * gain;
        }

        for (input, output) in zip(source_main, dest_main) {
            *output += *input * gain_vector;
        }

        for (input, output) in zip(source_post, dest_post) {
            *output += *input * gain;
        }
    } else {
        for (input, output) in zip(source, destination) {
            *output += *input * gain;
        }
    }
}

pub fn mix(source_1: &[f32], source_2: &[f32], destination: &mut [f32]) {
    debug_assert_eq!(source_1.len(), source_2.len());
    debug_assert_eq!(source_1.len(), destination.len());

    const VECTOR_SIZE: usize = 16;

    let (source_1_pre, source_1_main, source_1_post) = source_1.as_simd::<VECTOR_SIZE>();
    let (source_2_pre, source_2_main, source_2_post) = source_2.as_simd::<VECTOR_SIZE>();
    let (dest_pre, dest_main, dest_post) = destination.as_simd_mut::<VECTOR_SIZE>();

    if source_1_pre.len() == dest_pre.len() && source_1_pre.len() == source_2_pre.len() {
        for (input_1, input_2, output) in izip!(source_1_pre, source_2_pre, dest_pre) {
            *output = *input_1 + *input_2;
        }

        for (input_1, input_2, output) in izip!(source_1_main, source_2_main, dest_main) {
            *output = *input_1 + *input_2;
        }

        for (input_1, input_2, output) in izip!(source_1_post, source_2_post, dest_post) {
            *output = *input_1 + *input_2;
        }
    } else {
        for (input_1, input_2, output) in izip!(source_1, source_2, destination) {
            *output = *input_1 + *input_2;
        }
    }
}

pub fn subtract_from(source: &[f32], destination: &mut [f32]) {
    debug_assert_eq!(source.len(), destination.len());

    const VECTOR_SIZE: usize = 16;

    let (source_pre, source_main, source_post) = source.as_simd::<VECTOR_SIZE>();
    let (dest_pre, dest_main, dest_post) = destination.as_simd_mut::<VECTOR_SIZE>();

    if source_pre.len() == dest_pre.len() {
        for (input, output) in zip(source_pre, dest_pre) {
            *output -= *input;
        }

        for (input, output) in zip(source_main, dest_main) {
            *output -= *input;
        }

        for (input, output) in zip(source_post, dest_post) {
            *output -= *input;
        }
    } else {
        for (input, output) in zip(source, destination) {
            *output -= *input;
        }
    }
}

pub fn multiply(data: &mut [f32], gain: &[f32]) {
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

pub fn multiply_by_value(data: &mut [f32], gain: f32) {
    const VECTOR_SIZE: usize = 16;

    let (data_pre, data_main, data_post) = data.as_simd_mut::<VECTOR_SIZE>();
    let gain_vector = Simd::<f32, VECTOR_SIZE>::from_array([gain; VECTOR_SIZE]);

    for sample in data_pre {
        *sample *= gain;
    }

    for sample in data_main {
        *sample *= gain_vector;
    }

    for sample in data_post {
        *sample *= gain;
    }
}
