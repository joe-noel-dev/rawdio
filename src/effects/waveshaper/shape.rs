fn interpolate(a: f32, b: f32, amount_of_b: f32) -> f32 {
    (1.0 - amount_of_b) * a + amount_of_b * b
}

fn wave_shape_index_for_sample_value(sample: f32, shape_length: usize) -> f64 {
    let normalised = (sample as f64 + 1.0) / 2.0;
    let index = normalised * (shape_length as f64 - 1.0);
    index.clamp(0.0, (shape_length - 1) as f64)
}

fn shape_sample(sample: f32, wave_shape: &[f32]) -> f32 {
    debug_assert!(!wave_shape.is_empty());

    let shape_index = wave_shape_index_for_sample_value(sample, wave_shape.len());
    let index_before = shape_index.floor() as usize;
    let index_after = shape_index.ceil() as usize;

    if index_before == index_after {
        return *wave_shape.get(index_before).unwrap();
    }

    let shape_before = match wave_shape.get(index_before) {
        Some(value) => *value,
        None => return *wave_shape.last().unwrap(),
    };

    let shape_after = match wave_shape.get(index_after) {
        Some(value) => *value,
        None => return *wave_shape.last().unwrap(),
    };

    let amount_of_after = shape_index as f32 - index_before as f32;

    interpolate(shape_before, shape_after, amount_of_after)
}

pub fn shape(signal: &mut [f32], wave_shape: &[f32]) {
    signal
        .iter_mut()
        .for_each(|sample| *sample = shape_sample(*sample, wave_shape));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flat_shape_doesnt_affect_signal() {
        let wave_shape = [-1.0, 0.0, 1.0];
        let mut signal = [-1.0, -0.5, -0.25, 0.0, 0.25, 0.5, 1.0];

        shape(&mut signal, &wave_shape);

        assert_eq!(signal, [-1.0, -0.5, -0.25, 0.0, 0.25, 0.5, 1.0]);
    }

    #[test]
    fn inverted_shape_flips_magnitude() {
        let wave_shape = [1.0, 0.0, -1.0];
        let mut signal = [-1.0, -0.5, -0.25, 0.0, 0.25, 0.5, 1.0];

        shape(&mut signal, &wave_shape);

        assert_eq!(signal, [1.0, 0.5, 0.25, 0.0, -0.25, -0.5, -1.0]);
    }

    #[test]
    fn clipping() {
        let wave_shape = [-1.0, 0.0, 0.0];
        let mut signal = [-1.0, -0.5, -0.25, 0.0, 0.25, 0.5, 1.0];

        shape(&mut signal, &wave_shape);

        assert_eq!(signal, [-1.0, -0.5, -0.25, 0.0, 0.0, 0.0, 0.0]);
    }
}
