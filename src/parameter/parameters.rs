#![allow(dead_code)]

use std::collections::HashMap;

use crate::AudioParameter;

pub struct Parameters {
    params: HashMap<&'static str, AudioParameter>,
}

impl Parameters {
    pub fn empty() -> Self {
        Self {
            params: HashMap::new(),
        }
    }

    pub fn with_parameter(mut self, name: &'static str, param: AudioParameter) -> Self {
        self.add(name, param);
        self
    }

    pub fn add(&mut self, name: &'static str, param: AudioParameter) {
        self.params.insert(name, param);
    }

    pub fn get(&self, name: &str) -> Option<&AudioParameter> {
        self.params.get(name)
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut AudioParameter> {
        self.params.get_mut(name)
    }
}
