use crate::{parameter::ParameterRange, Level};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum CompressorParameter {
    Attack,
    Release,
    Ratio,
    Threshold,
    Knee,
    Makeup,
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
        CompressorParameter::Makeup => ParameterRange::new(
            Level::unity().as_gain(),
            Level::zero().as_gain(),
            Level::from_db(24.0).as_gain(),
        ),
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

pub fn attack_range() -> ParameterRange {
    get_range(CompressorParameter::Attack)
}

pub fn release_range() -> ParameterRange {
    get_range(CompressorParameter::Release)
}

pub fn ratio_range() -> ParameterRange {
    get_range(CompressorParameter::Ratio)
}

pub fn threshold_range() -> ParameterRange {
    get_range(CompressorParameter::Threshold)
}

pub fn knee_range() -> ParameterRange {
    get_range(CompressorParameter::Knee)
}

pub fn makeup_range() -> ParameterRange {
    get_range(CompressorParameter::Makeup)
}

pub fn wet_range() -> ParameterRange {
    get_range(CompressorParameter::WetLevel)
}

pub fn dry_range() -> ParameterRange {
    get_range(CompressorParameter::DryLevel)
}
