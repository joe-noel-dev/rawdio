use crate::{commands::Id, graph::DspProcessor, Level, SampleLocation};

use super::{biquad_coefficients::BiquadCoefficients, filter_type::BiquadFilterType};

pub struct BiquadProcessor {
    filter_type: BiquadFilterType,
    sample_rate: usize,
    frequency_id: Id,
    q_id: Id,
    shelf_gain_id: Id,
    gain_id: Id,
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
    filter_type: BiquadFilterType,
    parameters: &Parameters,
    sample_rate: usize,
) -> BiquadCoefficients {
    match filter_type {
        BiquadFilterType::HighPass => {
            BiquadCoefficients::high_pass(parameters.frequency, sample_rate as f64, parameters.q)
        }
        BiquadFilterType::LowPass => {
            BiquadCoefficients::low_pass(parameters.frequency, sample_rate as f64, parameters.q)
        }
        BiquadFilterType::BandPass => {
            BiquadCoefficients::band_pass(parameters.frequency, sample_rate as f64, parameters.q)
        }
        BiquadFilterType::Notch => {
            BiquadCoefficients::notch(parameters.frequency, sample_rate as f64, parameters.q)
        }
        BiquadFilterType::HighShelf => BiquadCoefficients::high_shelf(
            parameters.frequency,
            sample_rate as f64,
            Level::from_linear(parameters.shelf_gain),
        ),
        BiquadFilterType::LowShelf => BiquadCoefficients::low_shelf(
            parameters.frequency,
            sample_rate as f64,
            Level::from_linear(parameters.shelf_gain),
        ),
    }
}

impl BiquadProcessor {
    pub fn new(
        sample_rate: usize,
        channel_count: usize,
        filter_type: BiquadFilterType,
        frequency_id: Id,
        q_id: Id,
        shelf_gain_id: Id,
        gain_id: Id,
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
            gain_id,
            coefficients: calculate_coefficients(filter_type, &parameters, sample_rate),
            last_parameters: parameters,
            delays: (0..channel_count).map(|_| [0.0, 0.0, 0.0, 0.0]).collect(),
        }
    }
}

impl DspProcessor for BiquadProcessor {
    fn process_audio(&mut self, context: &mut crate::ProcessContext) {
        let frequency = context
            .parameters
            .get_parameter_values(self.frequency_id, context.output_buffer.frame_count());
        let q = context
            .parameters
            .get_parameter_values(self.q_id, context.output_buffer.frame_count());
        let shelf_gain = context
            .parameters
            .get_parameter_values(self.shelf_gain_id, context.output_buffer.frame_count());
        let gain = context
            .parameters
            .get_parameter_values(self.gain_id, context.output_buffer.frame_count());

        let frame_count = context.output_buffer.frame_count();
        let channel_count = context.output_buffer.channel_count();

        for frame in 0..frame_count {
            let parameters = Parameters {
                frequency: frequency[frame] as f64,
                q: q[frame] as f64,
                shelf_gain: shelf_gain[frame] as f64,
            };

            if parameters != self.last_parameters {
                self.coefficients =
                    calculate_coefficients(self.filter_type, &parameters, self.sample_rate);
                self.last_parameters = parameters;
            }

            for channel in 0..channel_count {
                let location = SampleLocation::new(channel, frame);
                let input_sample = context.input_buffer.get_sample(location) as f64;

                let x1 = self.delays[channel][0];
                let x2 = self.delays[channel][1];
                let y1 = self.delays[channel][2];
                let y2 = self.delays[channel][3];

                let out = self.coefficients.b0() * input_sample
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

                context.output_buffer.set_sample(location, out as f32);
            }
        }

        context.output_buffer.apply_gain(gain);
    }
}

fn denormal(sample: f64) -> f64 {
    let denormal_threshold = 1e-8;

    if -denormal_threshold <= sample && sample <= denormal_threshold {
        return 0.0;
    }

    sample
}
