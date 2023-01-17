use std::{thread, time};

use lockfree::channel::{spsc::Receiver, RecvErr};

use crate::graph::Dsp;

pub enum GarbageCollectionCommand {
    DisposeDsp(Box<Dsp>),
}

pub fn run_garbage_collector(mut receive_channel: Receiver<GarbageCollectionCommand>) {
    thread::spawn(move || loop {
        match receive_channel.recv() {
            Ok(command) => handle_garbage_collection_event(command),
            Err(RecvErr::NoMessage) => thread::sleep(time::Duration::from_secs(1)),
            Err(RecvErr::NoSender) => break,
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
