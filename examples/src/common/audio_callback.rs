#![allow(dead_code)]

use std::time::Duration;

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Device, Host, SampleFormat, Stream, SupportedStreamConfig,
};
use rawdio::{
    AudioBuffer, AudioProcess, BorrowedAudioBuffer, MutableBorrowedAudioBuffer, OwnedAudioBuffer,
    SampleLocation,
};

pub struct AudioCallback {
    input_channel_count: usize,
    output_stream: Stream,
    input_stream: Stream,
}

fn print_output_devices(host: &Host) {
    println!("Output devices: ");

    host.output_devices()
        .expect("Unable to access output devices")
        .for_each(|device| {
            let device_name = match device.name() {
                Ok(name) => name,
                Err(_) => return,
            };

            println!("  {device_name}");
        });
    println!();
}

fn print_input_devices(host: &Host) {
    println!("Input devices: ");

    host.input_devices()
        .expect("Unable to access input devices")
        .for_each(|device| {
            let device_name = match device.name() {
                Ok(name) => name,
                Err(_) => return,
            };

            println!("  {device_name}");
        });
    println!();
}

fn print_devices(host: &Host) {
    print_output_devices(host);
    print_input_devices(host);
}

type AudioSender = crossbeam::channel::Sender<f32>;
type AudioReceiver = crossbeam::channel::Receiver<f32>;

impl AudioCallback {
    pub fn new(audio_process: Box<dyn AudioProcess + Send>, sample_rate: usize) -> Self {
        let host = cpal::default_host();

        println!("Using audio host: {}\n", host.id().name());

        if cfg!(debug_assertions) {
            print_devices(&host);
        }

        let queue_capacity = 1024 * 1024;
        let (input_to_output_tx, input_to_output_rx) = crossbeam::channel::bounded(queue_capacity);

        let (input_stream, input_channel_count) =
            prepare_input_stream(&host, sample_rate, input_to_output_tx);

        let output_stream = prepare_output_stream(
            &host,
            sample_rate,
            input_channel_count,
            input_to_output_rx,
            audio_process,
        );

        Self {
            input_channel_count,
            output_stream,
            input_stream,
        }
    }

    pub fn input_channel_count(&self) -> usize {
        self.input_channel_count
    }
}

fn choose_input_config(device: &Device, sample_rate: usize) -> cpal::SupportedStreamConfig {
    let mut input_configs = device.supported_input_configs().unwrap();

    let cpal_sample_rate = cpal::SampleRate(sample_rate as u32);

    input_configs
        .find(|config| {
            config.min_sample_rate() <= cpal_sample_rate
                && cpal_sample_rate <= config.max_sample_rate()
                && config.sample_format() == SampleFormat::F32
        })
        .expect("No matching configurations for device")
        .with_sample_rate(cpal_sample_rate)
}

fn create_input_callback(
    config: SupportedStreamConfig,
    sample_rate: usize,
    input_to_output_tx: AudioSender,
) -> impl for<'a, 'b> Fn(&'a [f32], &'b cpal::InputCallbackInfo) {
    let channel_count = config.channels() as usize;

    let input_delay = Duration::from_millis(1);
    let input_delay =
        channel_count * (input_delay.as_secs_f64() * sample_rate as f64).ceil() as usize;

    (0..input_delay).for_each(|_| {
        let _ = input_to_output_tx.send(0.0_f32);
    });

    move |data: &[f32], _| {
        data.iter().for_each(|sample| {
            let _ = input_to_output_tx.send(*sample);
        });
    }
}

fn prepare_input_stream(
    host: &Host,
    sample_rate: usize,
    input_to_output_tx: AudioSender,
) -> (Stream, usize) {
    let preferred_device = host.default_input_device();
    let device = preferred_device.expect("Couldn't connect to input audio device");
    let config = choose_input_config(&device, sample_rate);

    println!("Connecting to input device: {}", device.name().unwrap());

    let input_callback = create_input_callback(config.clone(), sample_rate, input_to_output_tx);

    let input_error_callback = move |err| eprintln!("Input stream error: {err:?}");

    let input_stream = device
        .build_input_stream(&config.config(), input_callback, input_error_callback, None)
        .expect("Error creating input stream");

    input_stream.play().expect("Couldn't start input stream");

    (input_stream, config.channels() as usize)
}

fn create_output_callback(
    config: SupportedStreamConfig,
    input_channel_count: usize,
    input_to_output_rx: AudioReceiver,
    mut audio_process: Box<dyn AudioProcess + Send>,
) -> impl for<'a, 'b> FnMut(&'a mut [f32], &'b cpal::OutputCallbackInfo) {
    let max_buffer_size = match config.buffer_size() {
        cpal::SupportedBufferSize::Range { min: _, max } => *max as usize,
        cpal::SupportedBufferSize::Unknown => 4096,
    };

    let channel_count = config.channels() as usize;
    let sample_rate = config.sample_rate().0 as usize;

    let mut input_buffer = OwnedAudioBuffer::new(max_buffer_size, channel_count, sample_rate);

    let mut output_buffer = OwnedAudioBuffer::new(max_buffer_size, channel_count, sample_rate);

    move |data: &mut [f32], _| {
        let num_frames = data.len() / channel_count;

        (0..num_frames).for_each(|frame| {
            (0..input_channel_count).for_each(|channel| {
                let sample = input_to_output_rx
                    .try_recv()
                    .expect("Ran out of input samples");
                let location = SampleLocation::new(channel, frame);
                input_buffer.set_sample(location, sample);
            });
        });

        let input_slice = BorrowedAudioBuffer::slice_frames(&input_buffer, 0, num_frames);

        let mut output_slice =
            MutableBorrowedAudioBuffer::slice_frames(&mut output_buffer, 0, num_frames);

        output_slice.clear();

        audio_process.process(&input_slice, &mut output_slice);

        output_slice.copy_to_interleaved(data, channel_count, data.len() / channel_count);
    }
}

fn choose_output_config(device: &Device, sample_rate: usize) -> cpal::SupportedStreamConfig {
    let mut output_configs = device.supported_output_configs().unwrap();

    let cpal_sample_rate = cpal::SampleRate(sample_rate as u32);

    output_configs
        .find(|config| {
            config.min_sample_rate() <= cpal_sample_rate
                && cpal_sample_rate <= config.max_sample_rate()
                && config.sample_format() == SampleFormat::F32
        })
        .expect("No matching configurations for device")
        .with_sample_rate(cpal_sample_rate)
}

fn prepare_output_stream(
    host: &Host,
    sample_rate: usize,
    input_channel_count: usize,
    input_to_output_rx: AudioReceiver,
    audio_process: Box<dyn AudioProcess + Send>,
) -> Stream {
    let preferred_device = host.default_output_device();
    let device = preferred_device.expect("Couldn't connect to output audio device");
    let config = choose_output_config(&device, sample_rate);

    println!("Connecting to output device: {}", device.name().unwrap());

    let output_callback = create_output_callback(
        config.clone(),
        input_channel_count,
        input_to_output_rx,
        audio_process,
    );

    let output_error_callback = move |err| eprintln!("Output stream error: {err:?}");

    let stream = device
        .build_output_stream(
            &config.config(),
            output_callback,
            output_error_callback,
            None,
        )
        .expect("Couldn't create output stream");

    stream.play().expect("Couldn't start output stream");

    stream
}
