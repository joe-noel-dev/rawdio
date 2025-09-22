use approx::assert_relative_eq;
use std::time::Duration;
use rawdio::{prelude::*, Adsr};

struct Fixture {
    sample_rate: usize,
    channel_count: usize,
    context: Box<dyn Context>,
    audio_process: Box<dyn AudioProcess>,
    adsr: Adsr,
}

impl Fixture {
    fn process_duration(&mut self, duration: Duration) -> OwnedAudioBuffer {
        let frame_count = (duration.as_secs_f64() * self.sample_rate as f64).ceil() as usize;
        let input_buffer = OwnedAudioBuffer::sine(
            frame_count,
            self.channel_count,
            self.sample_rate,
            440.0, // A4
            1.0,   // Full amplitude
        );

        let mut output_buffer =
            OwnedAudioBuffer::new(frame_count, self.channel_count, self.sample_rate);

        self.audio_process
            .process(&input_buffer, &mut output_buffer);

        output_buffer
    }

    fn new(channel_count: usize) -> Self {
        let sample_rate = 48_000;

        let (mut context, process) =
            create_engine_with_options(EngineOptions::default().with_sample_rate(sample_rate));

        let adsr = Adsr::new(context.as_ref(), channel_count, sample_rate);

        connect_nodes!("input" => adsr => "output");

        context.start();

        Self {
            sample_rate,
            channel_count,
            context,
            audio_process: process,
            adsr,
        }
    }
}

impl Default for Fixture {
    fn default() -> Self {
        let channel_count = 1;
        Self::new(channel_count)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        self.context.stop();
    }
}

#[test]
fn test_adsr_envelope_shape() {
    let mut fixture = Fixture::default();
    
    // Set ADSR parameters
    let attack_time = Duration::from_millis(100);
    let decay_time = Duration::from_millis(150);
    let sustain_level = Level::from_db(-6.0); // 0.5 linear approximately
    let release_time = Duration::from_millis(200);
    
    fixture.adsr.set_adsr(attack_time, decay_time, sustain_level, release_time);
    
    // Trigger note on at start
    fixture.adsr.note_on_at_time(Timestamp::zero());
    
    // Process attack phase
    let attack_buffer = fixture.process_duration(attack_time);
    let attack_samples = attack_buffer.get_channel_data(SampleLocation::origin());
    
    // Check that attack phase is rising
    let max_attack = attack_samples.iter().fold(0.0f32, |a, &b| a.max(b.abs()));
    let min_attack = attack_samples.iter().fold(f32::INFINITY, |a, &b| a.min(b.abs()));
    
    assert!(max_attack > min_attack, "Attack phase should be rising");
    
    // Process decay phase
    let decay_buffer = fixture.process_duration(decay_time);
    let decay_samples = decay_buffer.get_channel_data(SampleLocation::origin());
    
    // Check that decay phase is falling from peak
    let max_decay = decay_samples.iter().fold(0.0f32, |a, &b| a.max(b.abs()));
    let min_decay = decay_samples.iter().fold(f32::INFINITY, |a, &b| a.min(b.abs()));
    
    assert!(max_decay > min_decay, "Decay phase should be falling");
    
    // Process sustain phase
    let sustain_duration = Duration::from_millis(300);
    let sustain_buffer = fixture.process_duration(sustain_duration);
    let sustain_samples = sustain_buffer.get_channel_data(SampleLocation::origin());
    
    // Check that sustain phase is relatively stable
    let sustain_mean = sustain_samples.iter().map(|&x| x.abs()).sum::<f32>() / sustain_samples.len() as f32;
    let sustain_variance = sustain_samples.iter()
        .map(|&x| (x.abs() - sustain_mean).powi(2))
        .sum::<f32>() / sustain_samples.len() as f32;
    
    // Sustain should be relatively stable (low variance)
    assert!(sustain_variance < 0.05, "Sustain phase should be stable, variance: {}", sustain_variance);
    
    // Trigger note off
    let note_off_time = Timestamp::from_seconds((attack_time + decay_time + sustain_duration).as_secs_f64());
    fixture.adsr.note_off_at_time(note_off_time);
    
    // Process release phase
    let release_buffer = fixture.process_duration(release_time);
    let release_samples = release_buffer.get_channel_data(SampleLocation::origin());
    
    // Check that release phase is falling to zero
    let max_release = release_samples.iter().fold(0.0f32, |a, &b| a.max(b.abs()));
    let min_release = release_samples.iter().fold(f32::INFINITY, |a, &b| a.min(b.abs()));
    
    assert!(max_release > min_release, "Release phase should be falling");
    
    // The end of release should be close to zero
    let end_samples = &release_samples[release_samples.len() - 10..];
    let end_average = end_samples.iter().map(|&x| x.abs()).sum::<f32>() / end_samples.len() as f32;
    assert!(end_average < 0.01, "Release should end near zero, got: {}", end_average);
}

#[test]
fn test_note_timing() {
    let mut fixture = Fixture::default();
    
    fixture.adsr.set_adsr(
        Duration::from_millis(50),  // short attack
        Duration::from_millis(50),  // short decay
        Level::from_db(-3.0),       // sustain level
        Duration::from_millis(100), // release
    );
    
    // Process some audio before note on - should be silent
    let pre_note_buffer = fixture.process_duration(Duration::from_millis(100));
    let pre_note_samples = pre_note_buffer.get_channel_data(SampleLocation::origin());
    let pre_note_energy = pre_note_samples.iter().map(|&x| x * x).sum::<f32>();
    
    assert_relative_eq!(pre_note_energy, 0.0, epsilon = 1e-6);
    
    // Trigger note on
    fixture.adsr.note_on_at_time(Timestamp::zero());
    
    // Process some audio after note on - should have signal
    let post_note_buffer = fixture.process_duration(Duration::from_millis(200));
    let post_note_samples = post_note_buffer.get_channel_data(SampleLocation::origin());
    let post_note_energy = post_note_samples.iter().map(|&x| x * x).sum::<f32>();
    
    assert!(post_note_energy > 0.001, "Should have signal after note on");
}

#[test]
fn test_parameter_changes() {
    let mut fixture = Fixture::default();
    
    // Start with one set of parameters
    fixture.adsr.set_attack_time(Duration::from_millis(200));
    fixture.adsr.set_decay_time(Duration::from_millis(100));
    fixture.adsr.set_sustain_level(Level::from_db(-12.0));
    fixture.adsr.set_release_time(Duration::from_millis(300));
    
    // Trigger a note
    fixture.adsr.note_on_at_time(Timestamp::zero());
    
    // Process some audio
    let buffer1 = fixture.process_duration(Duration::from_millis(100));
    let samples1 = buffer1.get_channel_data(SampleLocation::origin());
    let _max1 = samples1.iter().fold(0.0f32, |a, &b| a.max(b.abs()));
    
    // Change parameters
    fixture.adsr.set_attack_time(Duration::from_millis(50)); // Faster attack
    fixture.adsr.set_sustain_level(Level::from_db(-6.0));    // Higher sustain
    
    // Process more audio with new parameters
    let buffer2 = fixture.process_duration(Duration::from_millis(100));
    let samples2 = buffer2.get_channel_data(SampleLocation::origin());
    
    // Verify that the audio was processed (we can't easily verify the exact parameter changes
    // without more complex analysis, but we can verify the system still works)
    let max2 = samples2.iter().fold(0.0f32, |a, &b| a.max(b.abs()));
    assert!(max2 > 0.001, "Should still have signal after parameter changes");
}

#[test]
fn test_multichannel_adsr() {
    let channel_count = 2;
    let mut fixture = Fixture::new(channel_count);
    
    fixture.adsr.set_adsr(
        Duration::from_millis(100),
        Duration::from_millis(100),
        Level::from_db(-6.0),
        Duration::from_millis(150),
    );
    
    fixture.adsr.note_on_at_time(Timestamp::zero());
    
    let output_buffer = fixture.process_duration(Duration::from_millis(200));
    
    // Check that both channels have signal
    for channel in 0..channel_count {
        let channel_samples = output_buffer.get_channel_data(SampleLocation::channel(channel));
        let channel_energy = channel_samples.iter().map(|&x| x * x).sum::<f32>();
        assert!(channel_energy > 0.001, "Channel {} should have signal", channel);
    }
}