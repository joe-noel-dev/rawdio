use approx::assert_relative_eq;
use itertools::izip;
use rawdio::{
    create_engine_with_options, AudioBuffer, AudioProcess, Context, EngineOptions, Level, Mixer,
    OwnedAudioBuffer, SampleLocation,
};

struct Fixture {
    channel_count: usize,
    sample_rate: usize,
    context: Box<dyn Context>,
    audio_process: Box<dyn AudioProcess>,
    mixer: Mixer,
}

impl Fixture {
    fn process(&mut self, input: &dyn AudioBuffer) -> OwnedAudioBuffer {
        let mut output_buffer =
            OwnedAudioBuffer::new(input.frame_count(), self.channel_count, self.sample_rate);

        self.audio_process.process(input, &mut output_buffer);

        output_buffer
    }

    fn new(channel_count: usize) -> Self {
        let sample_rate = 44100;

        let (mut context, process) =
            create_engine_with_options(EngineOptions::default().with_sample_rate(sample_rate));

        let mixer = Mixer::new(context.as_ref(), channel_count, channel_count);

        mixer.node.connect_to_input();
        mixer.node.connect_to_output();

        context.start();

        Self {
            channel_count,
            sample_rate,
            context,
            audio_process: process,
            mixer,
        }
    }
}

impl Default for Fixture {
    fn default() -> Self {
        let channel_count = 2;
        Self::new(channel_count)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        self.context.stop();
    }
}

#[test]
fn test_phase_invert() {
    let channel_count = 1;
    let mut fixture = Fixture::new(channel_count);
    fixture.mixer.set_level(0, 0, Level::from_gain(-1.0));

    let frame_count = 1_024;
    let sample_rate = 48_000;
    let input_signal = OwnedAudioBuffer::white_noise(frame_count, channel_count, sample_rate);

    let output_signal = fixture.process(&input_signal);

    let input_data = input_signal.get_channel_data(SampleLocation::channel(0));
    let output_data = output_signal.get_channel_data(SampleLocation::channel(0));

    for (input_sample, output_sample) in izip!(input_data.iter(), output_data.iter()) {
        assert_relative_eq!(*output_sample, -1.0 * *input_sample);
    }
}

#[test]
fn test_channel_flip() {
    let channel_count = 2;
    let mut fixture = Fixture::new(channel_count);
    fixture.mixer.set_level(0, 0, Level::zero());
    fixture.mixer.set_level(0, 1, Level::unity());
    fixture.mixer.set_level(1, 0, Level::unity());
    fixture.mixer.set_level(1, 1, Level::zero());

    let frame_count = 1_024;
    let sample_rate = 48_000;
    let input_signal = OwnedAudioBuffer::white_noise(frame_count, channel_count, sample_rate);

    let output_signal = fixture.process(&input_signal);

    let input_0 = input_signal.get_channel_data(SampleLocation::channel(0));
    let input_1 = input_signal.get_channel_data(SampleLocation::channel(1));
    let output_0 = output_signal.get_channel_data(SampleLocation::channel(0));
    let output_1 = output_signal.get_channel_data(SampleLocation::channel(1));

    for (input_sample, output_sample) in izip!(input_0.iter(), output_1.iter()) {
        assert_relative_eq!(*output_sample, *input_sample);
    }

    for (input_sample, output_sample) in izip!(input_1.iter(), output_0.iter()) {
        assert_relative_eq!(*output_sample, *input_sample);
    }
}

#[test]
fn test_stereo_to_mono() {
    let channel_count = 2;
    let mut fixture = Fixture::new(channel_count);
    fixture.mixer.set_level(0, 0, Level::from_gain(0.5));
    fixture.mixer.set_level(1, 0, Level::from_gain(0.5));

    let frame_count = 1_024;
    let sample_rate = 48_000;
    let input_signal = OwnedAudioBuffer::white_noise(frame_count, channel_count, sample_rate);

    let output_signal = fixture.process(&input_signal);

    let input_0 = input_signal.get_channel_data(SampleLocation::channel(0));
    let input_1 = input_signal.get_channel_data(SampleLocation::channel(1));
    let output_0 = output_signal.get_channel_data(SampleLocation::channel(0));

    assert!(output_signal.channel_is_silent(1));

    for (input_sample_0, input_sample_1, output_sample) in
        izip!(input_0.iter(), input_1.iter(), output_0.iter())
    {
        let expected_sample = 0.5 * (*input_sample_0 + *input_sample_1);
        assert_relative_eq!(*output_sample, expected_sample);
    }
}
