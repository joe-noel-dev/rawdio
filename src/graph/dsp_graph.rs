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
    dsps: Pool<Id, RefCell<Dsp>>,
    output_connection: Option<OutputConnection>,
    garbase_collection_tx: Sender<GarbageCollectionCommand>,
}

impl DspGraph {
    pub fn process(&mut self, output_buffer: &mut dyn AudioBuffer, start_time: &Timestamp) {
        output_buffer.clear();

        let output_connection = match self.output_connection.clone() {
            Some(connection) => connection,
            None => return,
        };

        self.process_connection(&output_connection.from_id, output_buffer, start_time);
    }

    pub fn add_dsp(&mut self, dsp: RefCell<Dsp>) {
        let id = dsp.borrow().get_id();
        self.dsps.add(id, dsp);
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
            dsp.borrow_mut().request_parameter_change(change_request);
        }
    }

    pub fn add_connection(&mut self, connection: Connection) {
        if let Some(dsp) = self.dsps.get_mut(&connection.from_id) {
            dsp.borrow_mut().add_connection(connection);
        }
    }

    pub fn remove_connection(&mut self, connection: Connection) {
        if let Some(dsp) = self.dsps.get_mut(&connection.from_id) {
            dsp.borrow_mut().remove_connection(connection);
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
        let dsp = match self.dsps.get(dsp_id) {
            Some(dsp) => dsp,
            None => return,
        };

        dsp.borrow_mut().process_audio(output_buffer, start_time);
    }

    fn process_dependencies(
        &self,
        dsp_id: &Id,
        output_buffer: &mut dyn AudioBuffer,
        start_time: &Timestamp,
    ) {
        self.dsps
            .all()
            .filter(|(_, dsp)| dsp.borrow().is_connected_to(dsp_id))
            .for_each(|(id, _)| self.process_connection(id, output_buffer, start_time));
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
            dsps: Pool::new(64),
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

    fn make_dsp(frame_count: Arc<AtomicUsize>) -> RefCell<Dsp> {
        let processor = Box::new(Processor::new(frame_count));
        let parameters = DspParameterMap::new();
        let number_of_outputs = 1;
        RefCell::new(Dsp::new(
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
        let dsp_id = dsp.borrow().get_id();
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

    #[test]
    fn renders_chain() {
        let frame_count_1 = Arc::new(AtomicUsize::new(0));
        let frame_count_2 = Arc::new(AtomicUsize::new(0));

        let dsp_1 = make_dsp(frame_count_1.clone());
        let dsp_2 = make_dsp(frame_count_2.clone());

        let dsp_id_1 = dsp_1.borrow().get_id();
        let dsp_id_2 = dsp_2.borrow().get_id();

        let mut graph = DspGraph::default();

        graph.add_dsp(dsp_1);
        graph.add_dsp(dsp_2);

        let num_frames = 128;

        graph.connect_to_output(OutputConnection {
            from_id: dsp_id_2,
            output_index: 0,
        });

        graph.add_connection(Connection {
            from_id: dsp_id_1,
            output_index: 0,
            to_id: dsp_id_2,
            input_index: 0,
        });

        let mut audio_buffer = OwnedAudioBuffer::new(num_frames, 2, 44100);
        graph.process(&mut audio_buffer, &Timestamp::default());

        assert_eq!(frame_count_1.load(Ordering::Acquire), num_frames);
        assert_eq!(frame_count_2.load(Ordering::Acquire), num_frames);
    }
}
