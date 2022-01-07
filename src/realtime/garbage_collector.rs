use std::{thread, time};

use lockfree::channel::{spsc::Receiver, RecvErr};

use crate::{graph::dsp::Dsp, parameter::RealtimeAudioParameter};

pub enum GarbageCollectionCommand {
    DisposeDsp(Box<Dsp>),
    DisposeParameter(Box<RealtimeAudioParameter>),
}

pub fn run_garbage_collector(mut receive_channel: Receiver<GarbageCollectionCommand>) {
    thread::spawn(move || loop {
        match receive_channel.recv() {
            Ok(command) => handle_garabage_collection_event(command),
            Err(RecvErr::NoMessage) => thread::sleep(time::Duration::from_secs(1)),
            Err(RecvErr::NoSender) => break,
        };
    });
}

fn handle_garabage_collection_event(command: GarbageCollectionCommand) {
    match command {
        GarbageCollectionCommand::DisposeDsp(dsp) => {
            println!("Destroying DSP with ID: {:?}", dsp.get_id())
        }

        GarbageCollectionCommand::DisposeParameter(parameter) => println!(
            "Destroying audio parameter with ID: {:?}",
            parameter.get_id()
        ),
    }
}
