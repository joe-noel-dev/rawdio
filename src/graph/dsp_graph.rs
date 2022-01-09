use crate::{
    buffer::audio_buffer::AudioBuffer,
    commands::{command::ParameterChangeRequest, id::Id},
    timestamp::Timestamp,
    utility::{
        garbage_collector::{run_garbage_collector, GarbageCollectionCommand},
        pool::Pool,
    },
};

use lockfree::channel::{spsc, spsc::Sender};

use super::{
    connection::{Connection, OutputConnection},
    dsp::Dsp,
};

pub struct DspGraph {
    dsps: Pool<Id, Dsp>,
    output_connection: Option<OutputConnection>,
    garbase_collection_tx: Sender<GarbageCollectionCommand>,
}

impl DspGraph {
    pub fn process(&mut self, output_buffer: &mut dyn AudioBuffer, start_time: &Timestamp) {
        for (_, dsp) in self.dsps.all_mut() {
            dsp.process_audio(output_buffer, start_time);
        }
    }

    pub fn add_dsp(&mut self, dsp: Box<Dsp>) {
        self.dsps.add(dsp.get_id(), dsp);
    }

    pub fn remove_dsp(&mut self, id: Id) {
        if let Some(dsp) = self.dsps.remove(&id) {
            let _ = self
                .garbase_collection_tx
                .send(GarbageCollectionCommand::DisposeDsp(dsp));
        }
    }

    pub fn request_parameter_change(&mut self, change_request: ParameterChangeRequest) {
        if let Some(dsp) = self.dsps.get_mut(&change_request.dsp_id) {
            dsp.request_parameter_change(change_request);
        }
    }

    pub fn add_connection(&mut self, connection: Connection) {
        if let Some(dsp) = self.dsps.get_mut(&connection.from_id) {
            dsp.add_connection(connection);
        }
    }

    pub fn remove_connection(&mut self, connection: Connection) {
        if let Some(dsp) = self.dsps.get_mut(&connection.from_id) {
            dsp.remove_connection(connection);
        }
    }

    pub fn connect_to_output(&mut self, output_connection: OutputConnection) {
        self.output_connection = Some(output_connection);
    }
}

impl Default for DspGraph {
    fn default() -> Self {
        let (garbase_collection_tx, garbage_collection_rx) = spsc::create();
        run_garbage_collector(garbage_collection_rx);

        Self {
            dsps: Pool::new(64),
            output_connection: None,
            garbase_collection_tx,
        }
    }
}
