use crate::{commands::Id, graph::DspParameters, parameter::*, AudioParameter, Context};

type ParameterOptions = (&'static str, ParameterRange);

pub fn create_parameters<A>(
    dsp_id: Id,
    context: &dyn Context,
    options: A,
) -> (Parameters, DspParameters)
where
    A: IntoIterator<Item = ParameterOptions>,
{
    options
        .into_iter()
        .map(|(name, range)| (name, AudioParameter::new(dsp_id, name, range, context)))
        .fold(
            (Parameters::empty(), DspParameters::empty()),
            |(params, realtime_params), (name, (param, realtime_param))| {
                (
                    params.with_parameter(name, param),
                    realtime_params.with_parameter(realtime_param),
                )
            },
        )
}
