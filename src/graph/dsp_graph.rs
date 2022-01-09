use std::cell::RefCell;

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
    dsps: RefCell<Pool<Id, Dsp>>,
    output_connection: Option<OutputConnection>,
    garbase_collection_tx: Sender<GarbageCollectionCommand>,
}

impl DspGraph {
    pub fn process(&mut self, output_buffer: &mut dyn AudioBuffer, start_time: &Timestamp) {
        let output_connection = match self.output_connection.clone() {
            Some(connection) => connection,
            None => return,
        };

        self.process_connection(&output_connection.from_id, output_buffer, start_time);
    }

    pub fn add_dsp(&mut self, dsp: Box<Dsp>) {
        self.dsps.borrow_mut().add(dsp.get_id(), dsp);
    }

    pub fn remove_dsp(&mut self, id: Id) {
        if let Some(dsp) = self.dsps.borrow_mut().remove(&id) {
            let _ = self
                .garbase_collection_tx
                .send(GarbageCollectionCommand::DisposeDsp(dsp));
        }
    }

    pub fn request_parameter_change(&mut self, change_request: ParameterChangeRequest) {
        if let Some(dsp) = self.dsps.borrow_mut().get_mut(&change_request.dsp_id) {
            dsp.request_parameter_change(change_request);
        }
    }

    pub fn add_connection(&mut self, connection: Connection) {
        if let Some(dsp) = self.dsps.borrow_mut().get_mut(&connection.from_id) {
            dsp.add_connection(connection);
        }
    }

    pub fn remove_connection(&mut self, connection: Connection) {
        if let Some(dsp) = self.dsps.borrow_mut().get_mut(&connection.from_id) {
            dsp.remove_connection(connection);
        }
    }

    pub fn connect_to_output(&mut self, output_connection: OutputConnection) {
        self.output_connection = Some(output_connection);
    }

    fn process_dsp(
        &self,
        dsp_id: &Id,
        output_buffer: &mut dyn AudioBuffer,
        start_time: &Timestamp,
    ) {
        let mut dsps = self.dsps.borrow_mut();
        let dsp = match dsps.get_mut(dsp_id) {
            Some(dsp) => dsp,
            None => return,
        };

        dsp.process_audio(output_buffer, start_time);
    }

    fn process_dependencies(
        &self,
        dsp_id: &Id,
        output_buffer: &mut dyn AudioBuffer,
        start_time: &Timestamp,
    ) {
        if let Some(dsp) = self.dsps.borrow().get(dsp_id) {
            for connection in dsp.all_connections().flatten() {
                self.process_connection(&connection.from_id, output_buffer, start_time);
            }
        }
    }

    fn process_connection(
        &self,
        dsp_id: &Id,
        output_buffer: &mut dyn AudioBuffer,
        start_time: &Timestamp,
    ) {
        self.process_dependencies(dsp_id, output_buffer, start_time);
        self.process_dsp(dsp_id, output_buffer, start_time);
    }
}

impl Default for DspGraph {
    fn default() -> Self {
        let (garbase_collection_tx, garbage_collection_rx) = spsc::create();
        run_garbage_collector(garbage_collection_rx);

        Self {
            dsps: RefCell::new(Pool::new(64)),
            output_connection: None,
            garbase_collection_tx,
        }
    }
}

#[cfg(test)]
mod tests {

    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    use crate::{
        buffer::owned_audio_buffer::OwnedAudioBuffer,
        graph::dsp::{DspParameterMap, DspProcessor},
    };

    use super::*;

    struct Processor {
        frame_count: Arc<AtomicUsize>,
    }

    impl Processor {
        fn new(frame_count: Arc<AtomicUsize>) -> Self {
            Self { frame_count }
        }
    }

    impl DspProcessor for Processor {
        fn process_audio(
            &mut self,
            output_buffer: &mut dyn AudioBuffer,
            _start_time: &Timestamp,
            _parameters: &DspParameterMap,
        ) {
            self.frame_count.fetch_add(
                output_buffer.num_frames(),
                std::sync::atomic::Ordering::SeqCst,
            );
        }
    }

    fn make_dsp(frame_count: Arc<AtomicUsize>) -> Box<Dsp> {
        let processor = Box::new(Processor::new(frame_count));
        let parameters = DspParameterMap::new();
        let number_of_outputs = 1;
        Box::new(Dsp::new(
            Id::generate(),
            processor,
            parameters,
            number_of_outputs,
        ))
    }

    #[test]
    fn renders_when_connected_to_output() {
        let frame_count = Arc::new(AtomicUsize::new(0));
        let dsp = make_dsp(frame_count.clone());
        let dsp_id = dsp.get_id();
        let mut graph = DspGraph::default();
        graph.add_dsp(dsp);

        let num_frames = 128;

        let mut audio_buffer = OwnedAudioBuffer::new(num_frames, 2, 44100);
        graph.process(&mut audio_buffer, &Timestamp::default());

        assert_eq!(frame_count.load(Ordering::Acquire), 0);

        graph.connect_to_output(OutputConnection {
            from_id: dsp_id,
            output_index: 0,
        });

        graph.process(&mut audio_buffer, &Timestamp::default());

        assert_eq!(frame_count.load(Ordering::Acquire), num_frames);
    }
}
