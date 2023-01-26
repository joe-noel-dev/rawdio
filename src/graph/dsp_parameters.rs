use std::collections::HashMap;

use crate::{commands::Id, parameter::RealtimeAudioParameter};

pub struct DspParameters {
    parameters: HashMap<Id, RealtimeAudioParameter>,
}

impl DspParameters {
    pub fn new<const N: usize>(parameters: [RealtimeAudioParameter; N]) -> Self {
        Self {
            parameters: parameters
                .map(|parameter| (parameter.get_id(), parameter))
                .into(),
        }
    }

    pub fn empty() -> Self {
        Self {
            parameters: HashMap::new(),
        }
    }

    pub fn get_parameter(&self, id: Id) -> &RealtimeAudioParameter {
        self.parameters.get(&id).expect("Missing parameter")
    }

    pub fn get_parameter_mut(&mut self, id: Id) -> &mut RealtimeAudioParameter {
        self.parameters.get_mut(&id).expect("Missing parameter")
    }

    pub fn get_parameter_values(&self, id: Id) -> &[f64] {
        self.get_parameter(id).get_values()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&Id, &mut RealtimeAudioParameter)> {
        self.parameters.iter_mut()
    }
}
