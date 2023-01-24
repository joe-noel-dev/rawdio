use crate::Timestamp;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd)]
pub enum ValueChangeMethod {
    Immediate,
    Linear(Timestamp),
    Exponential(Timestamp),
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct ParameterChange {
    pub value: f64,
    pub end_time: Timestamp,
    pub method: ValueChangeMethod,
}
