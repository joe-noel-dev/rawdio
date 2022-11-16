use std::collections::HashMap;

use crate::{commands::Id, parameter::RealtimeAudioParameter};

pub type DspParameters = HashMap<Id, RealtimeAudioParameter>;
