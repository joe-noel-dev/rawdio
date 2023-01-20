use crate::{commands::Id, graph::DspProcessor, Level, SampleLocation};

use super::{biquad_coefficients::BiquadCoefficients, filter_type::FilterType};

pub struct BiquadProcessor {
    filter_type: FilterType,
    sample_rate: usize,
    frequency_id: Id,
    q_id: Id,
    shelf_gain_id: Id,
    coefficients: BiquadCoefficients,
    last_parameters: Parameters,
    delays: Vec<[f64; 4]>,
}

#[derive(PartialEq)]
struct Parameters {
    frequency: f64,
    q: f64,
    shelf_gain: f64,
}

fn calculate_coefficients(
    filter_type: FilterType,
    parameters: &Parameters,
    sample_rate: usize,
) -> BiquadCoefficients {
    match filter_type {
        FilterType::HighPass => {
            BiquadCoefficients::high_pass(parameters.frequency, sample_rate as f64, parameters.q)
        }
        FilterType::LowPass => {
            BiquadCoefficients::low_pass(parameters.frequency, sample_rate as f64, parameters.q)
        }
        FilterType::BandPass => {
            BiquadCoefficients::band_pass(parameters.frequency, sample_rate as f64, parameters.q)
        }
        FilterType::Notch => {
            BiquadCoefficients::notch(parameters.frequency, sample_rate as f64, parameters.q)
        }
        FilterType::HighShelf => BiquadCoefficients::high_shelf(
            parameters.frequency,
            sample_rate as f64,
            Level::from_gain(parameters.shelf_gain),
        ),
        FilterType::LowShelf => BiquadCoefficients::low_shelf(
            parameters.frequency,
            sample_rate as f64,
            Level::from_gain(parameters.shelf_gain),
        ),
    }
}

impl BiquadProcessor {
    pub fn new(
        sample_rate: usize,
        num_channels: usize,
        filter_type: FilterType,
        frequency_id: Id,
        q_id: Id,
        shelf_gain_id: Id,
    ) -> Self {
        let parameters = Parameters {
            frequency: 1_000.0,
            q: 1.0 / 2.0_f64.sqrt(),
            shelf_gain: 1.0,
        };

        Self {
            sample_rate,
            filter_type,
            frequency_id,
            q_id,
            shelf_gain_id,
            coefficients: calculate_coefficients(filter_type, &parameters, sample_rate),
            last_parameters: parameters,
            delays: (0..num_channels).map(|_| [0.0, 0.0, 0.0, 0.0]).collect(),
        }
    }
}

macro_rules! unwrap_or_return {
    ( $e:expr ) => {
        match $e {
            Some(x) => x,
            None => return,
        }
    };
}

impl DspProcessor for BiquadProcessor {
    fn process_audio(
        &mut self,
        input_buffer: &dyn crate::AudioBuffer,
        output_buffer: &mut dyn crate::AudioBuffer,
        _start_time: &crate::timestamp::Timestamp,
        parameters: &crate::graph::DspParameters,
    ) {
        let frequency = unwrap_or_return!(parameters.get(&self.frequency_id)).get_values();
        let q = unwrap_or_return!(parameters.get(&self.q_id)).get_values();
        let shelf_gain = unwrap_or_return!(parameters.get(&self.shelf_gain_id)).get_values();

        let frame_count = output_buffer.frame_count();
        let channel_count = output_buffer.channel_count();

        for frame in 0..frame_count {
            let parameters = Parameters {
                frequency: frequency[frame],
                q: q[frame],
                shelf_gain: shelf_gain[frame],
            };

            if parameters != self.last_parameters {
                self.coefficients =
                    calculate_coefficients(self.filter_type, &parameters, self.sample_rate);
                self.last_parameters = parameters;
            }

            for channel in 0..channel_count {
                let location = SampleLocation::new(channel, frame);
                let input_sample = input_buffer.get_sample(location) as f64;

                let x1 = self.delays[channel][0];
                let x2 = self.delays[channel][1];
                let y1 = self.delays[channel][2];
                let y2 = self.delays[channel][3];

                let out = self.coefficients.b0() * input_sample as f64
                    + self.coefficients.b1() * x1
                    + self.coefficients.b2() * x2
                    - self.coefficients.a1() * y1
                    - self.coefficients.a2() * y2;

                self.delays[channel][1] = x1;
                self.delays[channel][0] = input_sample;
                self.delays[channel][3] = y1;
                self.delays[channel][2] = out;

                for delay in self.delays[channel].iter_mut() {
                    *delay = denormal(*delay);
                }

                output_buffer.set_sample(location, out as f32);
            }
        }
    }
}

fn denormal(sample: f64) -> f64 {
    let denormal_threshold = 1e-8;

    if -denormal_threshold <= sample && sample <= denormal_threshold {
        return 0.0;
    }

    sample
}
