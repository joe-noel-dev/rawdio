use std::time::Duration;

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Host, InputCallbackInfo, SampleFormat, Stream,
};
use rawdio::{
    AudioBuffer, AudioProcess, BorrowedAudioBuffer, MutableBorrowedAudioBuffer, OwnedAudioBuffer,
    SampleLocation,
};

pub struct AudioCallback {
    input_channel_count: usize,
    _output_stream: Stream,
    _input_stream: Stream,
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
            _output_stream: output_stream,
            _input_stream: input_stream,
        }
    }

    pub fn input_channel_count(&self) -> usize {
        self.input_channel_count
    }
}

fn prepare_input_stream(
    host: &Host,
    sample_rate: usize,
    input_to_output_tx: AudioSender,
) -> (Stream, usize) {
    let preferred_device = host.default_input_device();

    let device = preferred_device.expect("Couldn't connect to input audio device");

    let mut input_configs = device.supported_input_configs().unwrap();

    let cpal_sample_rate = cpal::SampleRate(sample_rate as u32);

    let config = input_configs
        .find(|config| {
            config.min_sample_rate() <= cpal_sample_rate
                && cpal_sample_rate <= config.max_sample_rate()
                && config.sample_format() == SampleFormat::F32
        })
        .expect("No matching configurations for device")
        .with_sample_rate(cpal_sample_rate);

    println!("Connecting to input device: {}", device.name().unwrap());

    let channel_count = config.channels() as usize;

    let input_delay = Duration::from_millis(1);
    let input_delay =
        channel_count * (input_delay.as_secs_f64() * sample_rate as f64).ceil() as usize;

    (0..input_delay).for_each(|_| {
        let _ = input_to_output_tx.send(0.0_f32);
    });

    let input_callback = move |data: &[f32], _: &InputCallbackInfo| {
        data.iter().for_each(|sample| {
            let _ = input_to_output_tx.send(*sample);
        });
    };

    let input_error_callback = move |err| eprintln!("Input stream error: {err:?}");

    let input_stream = device
        .build_input_stream(&config.config(), input_callback, input_error_callback)
        .expect("Error creating input stream");

    input_stream.play().expect("Couldn't start input stream");

    (input_stream, channel_count)
}

fn prepare_output_stream(
    host: &Host,
    sample_rate: usize,
    input_channel_count: usize,
    input_to_output_rx: AudioReceiver,
    mut audio_process: Box<dyn AudioProcess + Send>,
) -> Stream {
    let preferred_device = host.default_output_device();

    let device = preferred_device.expect("Couldn't connect to output audio device");

    let mut output_configs = device.supported_output_configs().unwrap();

    let cpal_sample_rate = cpal::SampleRate(sample_rate as u32);
    let config = output_configs
        .find(|config| {
            config.min_sample_rate() <= cpal_sample_rate
                && cpal_sample_rate <= config.max_sample_rate()
                && config.sample_format() == SampleFormat::F32
        })
        .expect("No matching configurations for device")
        .with_sample_rate(cpal_sample_rate);

    println!("Connecting to output device: {}", device.name().unwrap());

    let max_buffer_size = match config.buffer_size() {
        cpal::SupportedBufferSize::Range { min: _, max } => *max,
        cpal::SupportedBufferSize::Unknown => 4096,
    };

    let channel_count = config.channels() as usize;

    let mut input_buffer = OwnedAudioBuffer::new(
        max_buffer_size as usize,
        channel_count,
        config.sample_rate().0 as usize,
    );

    let mut output_buffer = OwnedAudioBuffer::new(
        max_buffer_size as usize,
        channel_count,
        config.sample_rate().0 as usize,
    );

    let output_callback = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
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
    };

    let output_error_callback = move |err| eprintln!("Output stream error: {err:?}");

    let stream = device
        .build_output_stream(&config.config(), output_callback, output_error_callback)
        .expect("Couldn't create output stream");

    stream.play().expect("Couldn't start output stream");

    stream
}
