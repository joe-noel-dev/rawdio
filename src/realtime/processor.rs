use crate::{
    audio_process::AudioProcess,
    buffer::audio_buffer::AudioBuffer,
    commands::{
        command::{Command, ParameterChangeRequest},
        id::Id,
        notification::Notification,
    },
    graph::dsp::Dsp,
    timestamp::Timestamp,
    utility::pool::Pool,
};
use lockfree::channel::{
    mpsc::Receiver,
    spsc::{self, Sender},
};

use super::garbage_collector::{run_garbage_collector, GarbageCollectionCommand};

const MAXIMUM_NUMBER_OF_FRAMES: usize = 1024;
const MAXIMUM_NUMBER_OF_CHANNELS: usize = 2;

pub struct Processor {
    started: bool,
    sample_rate: usize,
    command_rx: Receiver<Command>,
    notification_tx: Sender<Notification>,
    garbase_collection_tx: Sender<GarbageCollectionCommand>,
    sample_position: usize,
    dsps: Pool<Id, Dsp>,
}

impl Processor {
    pub fn new(
        sample_rate: usize,
        command_rx: Receiver<Command>,
        notification_tx: Sender<Notification>,
    ) -> Self {
        let (garbase_collection_tx, garbage_collection_rx) = spsc::create();

        run_garbage_collector(garbage_collection_rx);

        Self {
            started: false,
            sample_rate,
            command_rx,
            notification_tx,
            garbase_collection_tx,
            sample_position: 0,

            dsps: Pool::new(64),
        }
    }
}

impl AudioProcess for Processor {
    fn process(&mut self, data: &mut dyn AudioBuffer) {
        assert!(data.num_frames() <= self.get_maximum_number_of_frames());

        data.clear();

        self.process_commands();

        if !self.started {
            return;
        }

        let current_time = self.current_time();

        for (_, dsp) in self.dsps.all_mut() {
            dsp.process_audio(data, &current_time);
        }

        self.update_position(data.num_frames());
        self.notify_position();
    }

    fn get_maximum_number_of_frames(&self) -> usize {
        MAXIMUM_NUMBER_OF_FRAMES
    }

    fn get_maximum_number_of_channel(&self) -> usize {
        MAXIMUM_NUMBER_OF_CHANNELS
    }
}

impl Processor {
    fn process_commands(&mut self) {
        while let Ok(command) = self.command_rx.recv() {
            match command {
                Command::Start => self.started = true,
                Command::Stop => self.started = false,
                Command::AddDsp(dsp) => self.add_dsp(dsp),
                Command::RemoveDsp(id) => self.remove_dsp(id),

                Command::ParameterValueChange(change_request) => {
                    self.request_parameter_change(change_request)
                }
            }
        }
    }

    fn send_notficiation(&mut self, notification: Notification) {
        let _ = self.notification_tx.send(notification);
    }

    fn update_position(&mut self, num_samples: usize) {
        self.sample_position += num_samples;
    }

    fn current_time(&self) -> Timestamp {
        Timestamp::from_seconds(self.sample_position as f64 / self.sample_rate as f64)
    }

    fn notify_position(&mut self) {
        self.send_notficiation(Notification::Position(self.current_time()));
    }

    fn add_dsp(&mut self, dsp: Box<Dsp>) {
        self.dsps.add(dsp.get_id(), dsp);
    }

    fn remove_dsp(&mut self, id: Id) {
        if let Some(dsp) = self.dsps.remove(&id) {
            let _ = self
                .garbase_collection_tx
                .send(GarbageCollectionCommand::DisposeDsp(dsp));
        }
    }

    fn request_parameter_change(&mut self, change_request: ParameterChangeRequest) {
        if let Some(dsp) = self.dsps.get_mut(&change_request.dsp_id) {
            dsp.request_parameter_change(change_request);
        }
    }
}
