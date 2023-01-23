mod assigned_buffer_pool;
mod connection;
mod dsp;
mod dsp_parameters;
mod endpoint;
mod graph_node;

pub type AssignedBufferPool<Identifier> = assigned_buffer_pool::AssignedBufferPool<Identifier>;
pub type Connection = connection::Connection;
pub type Dsp = dsp::Dsp;
pub type Endpoint = endpoint::Endpoint;
pub type EndpointType = endpoint::EndpointType;
pub type DspParameters = dsp_parameters::DspParameters;

pub use dsp::DspProcessor;
pub use graph_node::GraphNode;
