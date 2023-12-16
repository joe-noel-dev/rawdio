use crate::{parameter::ParameterRange, Level};

pub fn get_range(parameter: &'static str) -> ParameterRange {
    match parameter {
        "attack" => ParameterRange::new(1.0, 0.0, 1_000.0),
        "release" => ParameterRange::new(10.0, 0.0, 10_000.0),
        "ratio" => ParameterRange::new(3.0, 1.0, f64::MAX),
        "threshold" => ParameterRange::new(0.0, -128.0, 24.0),
        "knee" => ParameterRange::new(0.0, 0.0, 24.0),
        "wet" => ParameterRange::new(
            Level::unity().as_linear(),
            Level::zero().as_linear(),
            Level::from_db(12.0).as_linear(),
        ),
        "dry" => ParameterRange::new(
            Level::zero().as_linear(),
            Level::zero().as_linear(),
            Level::from_db(12.0).as_linear(),
        ),
        _ => panic!("Unsupported parameter: {parameter}"),
    }
}
