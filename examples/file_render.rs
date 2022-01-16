use rust_audio_engine::{
    buffer::{
        audio_buffer::AudioBuffer, audio_buffer_slice::AudioBufferSlice,
        owned_audio_buffer::OwnedAudioBuffer, sample_location::SampleLocation,
    },
    context::Context,
    graph::node::Node,
    nodes::{gain::GainNode, oscillator::OscillatorNode},
    timestamp::Timestamp,
};

fn main() {
    let sample_rate = 44100;
    let mut context = Context::new(sample_rate);
    let mut audio_process = context.get_audio_process();

    let mut oscillator_1 = OscillatorNode::new(context.get_command_queue(), 440.0);
    oscillator_1
        .gain
        .set_value_at_time(0.8, Timestamp::from_seconds(0.0));

    let mut oscillator_2 = OscillatorNode::new(context.get_command_queue(), 880.0);
    oscillator_2
        .gain
        .set_value_at_time(0.4, Timestamp::from_seconds(0.0));

    let mut oscillator_3 = OscillatorNode::new(context.get_command_queue(), 1320.0);
    oscillator_3
        .gain
        .set_value_at_time(0.2, Timestamp::from_seconds(0.0));

    let mut oscillator_4 = OscillatorNode::new(context.get_command_queue(), 1760.0);
    oscillator_4
        .gain
        .set_value_at_time(0.1, Timestamp::from_seconds(0.0));

    let mut gain = GainNode::new(context.get_command_queue());

    oscillator_1.connect_to(gain.get_id());
    oscillator_2.connect_to(gain.get_id());
    oscillator_3.connect_to(gain.get_id());
    oscillator_4.connect_to(gain.get_id());

    gain.connect_to_output();

    gain.gain
        .set_value_at_time(0.9, Timestamp::from_seconds(0.0));

    gain.gain
        .linear_ramp_to_value(0.0, Timestamp::from_seconds(4.0));

    context.start();

    let file_spec = hound::WavSpec {
        channels: 2,
        sample_rate: sample_rate as u32,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::create("output.wav", file_spec).unwrap();

    let mut audio_buffer = OwnedAudioBuffer::new(sample_rate * 4, 2, sample_rate);

    let mut offset = 0;

    while offset < audio_buffer.num_frames() {
        let num_frames = std::cmp::min(
            audio_buffer.num_frames() - offset,
            audio_process.get_maximum_number_of_frames(),
        );

        let mut audio_buffer = AudioBufferSlice::new(&mut audio_buffer, offset, num_frames);
        audio_process.process(&mut audio_buffer);

        offset += num_frames;
    }

    for frame in 0..audio_buffer.num_frames() {
        for channel in 0..audio_buffer.num_channels() {
            let sample = audio_buffer.get_sample(&SampleLocation::new(channel, frame));
            let scale = i16::MAX as f32;
            let sample = (sample * scale) as i16;
            writer.write_sample(sample).expect("Failed to write sample");
        }
    }

    context.stop();
}
