use std::sync::Arc;

use atomic_float::AtomicF64;

pub type ParameterValue = Arc<AtomicF64>;
