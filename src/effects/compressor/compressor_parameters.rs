use crate::{parameter::ParameterRange, Level};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum CompressorParameter {
    Attack,
    Release,
    Ratio,
    Threshold,
    Knee,
    WetLevel,
    DryLevel,
}

pub fn get_range(parameter: CompressorParameter) -> ParameterRange {
    match parameter {
        CompressorParameter::Attack => ParameterRange::new(1.0, 0.0, 1_000.0),
        CompressorParameter::Release => ParameterRange::new(10.0, 0.0, 10_000.0),
        CompressorParameter::Ratio => ParameterRange::new(3.0, 1.0, f64::MAX),
        CompressorParameter::Threshold => ParameterRange::new(0.0, -128.0, 24.0),
        CompressorParameter::Knee => ParameterRange::new(0.0, 0.0, 24.0),
        CompressorParameter::WetLevel => ParameterRange::new(
            Level::unity().as_gain(),
            Level::zero().as_gain(),
            Level::from_db(12.0).as_gain(),
        ),
        CompressorParameter::DryLevel => ParameterRange::new(
            Level::zero().as_gain(),
            Level::zero().as_gain(),
            Level::from_db(12.0).as_gain(),
        ),
    }
}
