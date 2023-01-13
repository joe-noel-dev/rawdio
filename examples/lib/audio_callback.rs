use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Host, SampleFormat, Stream,
};
use rust_audio_engine::{AudioBuffer, AudioProcess, BorrowedAudioBuffer, OwnedAudioBuffer};

pub struct AudioCallback {
    _output_stream: Stream,
}

fn print_output_devices(host: &Host) {
    println!("Output devices: ");
    host.output_devices().unwrap().for_each(|device| {
        let device_name = match device.name() {
            Ok(name) => name,
            Err(_) => return,
        };

        println!("{device_name}");
    });
    println!();
}

impl AudioCallback {
    pub fn new(mut audio_process: Box<dyn AudioProcess + Send>, sample_rate: usize) -> Self {
        let host = cpal::default_host();
        println!("Using audio host: {}\n", host.id().name());

        print_output_devices(&host);

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

        println!("Connecting to device: {}", device.name().unwrap());
        println!("Sample rate: {}\n", config.sample_rate().0);

        let max_buffer_size = match config.buffer_size() {
            cpal::SupportedBufferSize::Range { min: _, max } => *max,
            cpal::SupportedBufferSize::Unknown => 4096,
        };

        let mut audio_buffer = OwnedAudioBuffer::new(
            max_buffer_size as usize,
            config.channels() as usize,
            config.sample_rate().0 as usize,
        );

        let stream = device
            .build_output_stream(
                &config.config(),
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let num_frames = data.len() / config.channels() as usize;
                    let mut slice =
                        BorrowedAudioBuffer::slice_frames(&mut audio_buffer, 0, num_frames);

                    slice.clear();

                    audio_process.process(&mut slice);

                    slice.copy_to_interleaved(
                        data,
                        config.channels() as usize,
                        data.len() / config.channels() as usize,
                    );
                },
                move |err| eprintln!("Stream error: {err:?}"),
            )
            .expect("Couldn't create output stream");

        stream.play().expect("Couldn't start output stream");

        Self {
            _output_stream: stream,
        }
    }
}
