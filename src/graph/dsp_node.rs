use crate::{AudioParameter, Parameters};

/// A DSP node in the graph that has parameters
pub trait DspNode {
    /// Get the parameters for the node
    fn get_parameters_mut(&mut self) -> &mut Parameters;

    /// Get a single parameter by name
    ///
    /// Panics if the node doesn't have a parameter with that name
    fn get_parameter_mut(&mut self, parameter: &str) -> &mut AudioParameter {
        self.get_parameters_mut()
            .get_mut(parameter)
            .unwrap_or_else(|| panic!("Parameter not found: {parameter}"))
    }
}
