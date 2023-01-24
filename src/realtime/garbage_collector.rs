use std::{thread, time};

use crossbeam::channel::TryRecvError;

use crate::graph::Dsp;

pub type GarbaseCollectionSender = crossbeam::channel::Sender<GarbageCollectionCommand>;
pub type GarbaseCollectionReceiver = crossbeam::channel::Receiver<GarbageCollectionCommand>;

pub enum GarbageCollectionCommand {
    DisposeDsp(Box<Dsp>),
}

pub fn run_garbage_collector(receive_channel: GarbaseCollectionReceiver) {
    thread::spawn(move || loop {
        match receive_channel.try_recv() {
            Ok(command) => handle_garbage_collection_event(command),
            Err(TryRecvError::Empty) => thread::sleep(time::Duration::from_secs(1)),
            Err(TryRecvError::Disconnected) => break,
        };
    });
}

fn handle_garbage_collection_event(command: GarbageCollectionCommand) {
    match command {
        GarbageCollectionCommand::DisposeDsp(dsp) => {
            println!("Destroying DSP with ID: {:?}", dsp.get_id())
        }
    }
}
