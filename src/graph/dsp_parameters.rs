use crate::parameter::*;
use std::collections::HashMap;

pub struct DspParameters {
    parameters: HashMap<ParameterId, RealtimeAudioParameter>,
}

impl DspParameters {
    pub fn new<I>(parameters: I) -> Self
    where
        I: IntoIterator<Item = RealtimeAudioParameter>,
    {
        Self {
            parameters: parameters
                .into_iter()
                .map(|parameter| (parameter.get_id(), parameter))
                .collect(),
        }
    }

    pub fn with_parameter(mut self, parameter: RealtimeAudioParameter) -> Self {
        self.parameters.insert(parameter.get_id(), parameter);
        self
    }

    pub fn empty() -> Self {
        Self {
            parameters: HashMap::new(),
        }
    }

    pub fn get_parameter(&self, id: ParameterId) -> &RealtimeAudioParameter {
        self.parameters.get(&id).expect("Missing parameter")
    }

    pub fn get_parameter_mut(&mut self, id: ParameterId) -> &mut RealtimeAudioParameter {
        self.parameters.get_mut(&id).expect("Missing parameter")
    }

    pub fn get_parameter_values(&self, id: ParameterId, frame_count: usize) -> &[f32] {
        self.get_parameter(id).get_values(frame_count)
    }

    pub fn iter_mut(
        &mut self,
    ) -> impl Iterator<Item = (&ParameterId, &mut RealtimeAudioParameter)> {
        self.parameters.iter_mut()
    }
}
