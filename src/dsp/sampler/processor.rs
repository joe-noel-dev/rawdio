use crate::{
    graph::dsp::{DspParameterMap, DspProcessor},
    AudioBuffer, AudioBufferSlice, OwnedAudioBuffer, Timestamp,
};

use super::{fade::Fade, voice::Voice};

pub type EventReceiver = lockfree::channel::spsc::Receiver<SamplerEvent>;
pub type EventTransmitter = lockfree::channel::spsc::Sender<SamplerEvent>;

pub struct SamplerDspProcess {
    fade: Fade,
    voices: Vec<Voice>,
    active_voice: Option<usize>,
    buffer: OwnedAudioBuffer,
    event_receiver: EventReceiver,
    pending_events: Vec<SamplerEvent>,
}

const NUM_VOICES: usize = 2;
const FADE_LENGTH_MS: f64 = 50.0;
const MAX_PENDING_EVENTS: usize = 10;

pub enum SampleEventType {
    Start(Timestamp),
    Stop,
}

pub struct SamplerEvent {
    time: Timestamp,
    event_type: SampleEventType,
}

impl SamplerEvent {
    pub fn start(start_at_time: Timestamp, position_in_sample: Timestamp) -> Self {
        Self {
            time: start_at_time,
            event_type: SampleEventType::Start(position_in_sample),
        }
    }

    pub fn stop(stop_at_time: Timestamp) -> Self {
        Self {
            time: stop_at_time,
            event_type: SampleEventType::Stop,
        }
    }
}

impl DspProcessor for SamplerDspProcess {
    fn process_audio(
        &mut self,
        _input_buffer: &dyn AudioBuffer,
        output_buffer: &mut dyn AudioBuffer,
        start_time: &Timestamp,
        _parameters: &DspParameterMap,
    ) {
        self.read_events();

        let mut current_time = *start_time;
        let end_time = start_time
            .incremented_by_samples(output_buffer.num_frames(), output_buffer.sample_rate());

        let sample_rate = output_buffer.sample_rate();

        let mut position = 0;
        while position < output_buffer.num_frames() {
            let end_frame = if let Some(next_event) = self.next_event_before(&end_time) {
                let event_time = std::cmp::max(next_event.time, current_time);
                self.process_event(&next_event, sample_rate);
                let position_in_frame = event_time - *start_time;
                position_in_frame.get_samples(sample_rate).floor() as usize
            } else {
                output_buffer.num_frames()
            };

            let num_frames = end_frame - position;

            debug_assert!(num_frames <= output_buffer.num_frames() - position);

            let mut output_buffer = AudioBufferSlice::new(output_buffer, position, num_frames);
            self.process_voices(&mut output_buffer);

            position += num_frames;
            current_time = current_time.incremented_by_samples(num_frames, sample_rate);
        }
    }
}

impl SamplerDspProcess {
    pub fn new(
        sample_rate: usize,
        buffer: OwnedAudioBuffer,
        event_receiver: EventReceiver,
    ) -> Self {
        Self {
            fade: Fade::new(FADE_LENGTH_MS, sample_rate),
            voices: (0..NUM_VOICES).map(|_| Voice::default()).collect(),
            active_voice: None,
            buffer,
            event_receiver,
            pending_events: Vec::with_capacity(MAX_PENDING_EVENTS),
        }
    }

    pub fn process_voices(&mut self, output_buffer: &mut dyn AudioBuffer) {
        let fade = &self.fade;
        let sample = &self.buffer;
        self.voices
            .iter_mut()
            .for_each(|voice| voice.render(output_buffer, sample, fade));
    }

    fn next_event_before(&mut self, end_time: &Timestamp) -> Option<SamplerEvent> {
        if let Some(next_event) = self.pending_events.first() {
            if next_event.time < *end_time {
                return self.pending_events.pop();
            }
        }

        None
    }

    fn process_event(&mut self, event: &SamplerEvent, sample_rate: usize) {
        match event.event_type {
            SampleEventType::Start(position_in_sample) => {
                let position_in_sample = position_in_sample.get_samples(sample_rate);
                self.start(position_in_sample as usize);
            }
            SampleEventType::Stop => self.stop(),
        }
    }

    fn read_events(&mut self) {
        let mut sort_required = false;

        while let Ok(event) = self.event_receiver.recv() {
            self.pending_events.push(event);
            sort_required = true;
        }

        if sort_required {
            self.pending_events
                .sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap())
        }
    }

    fn assign_voice(&mut self, position: usize) {
        self.stop();

        if let Some((index, free_voice)) = self
            .voices
            .iter_mut()
            .enumerate()
            .find(|(_, voice)| voice.is_stopped())
        {
            free_voice.start_from_position(position);
            self.active_voice = Some(index);
        }
    }

    fn get_active_voice(&self) -> Option<&Voice> {
        if let Some(active_voice_index) = self.active_voice {
            return self.voices.get(active_voice_index);
        }

        None
    }

    fn get_active_voice_position(&self) -> Option<usize> {
        self.get_active_voice().map(|voice| voice.get_position())
    }

    fn start(&mut self, from_position: usize) {
        if let Some(current_position) = self.get_active_voice_position() {
            if current_position == from_position {
                return;
            }
        }

        self.assign_voice(from_position);
    }

    fn stop(&mut self) {
        self.voices.iter_mut().for_each(|voice| voice.stop());
        self.active_voice = None
    }
}

#[cfg(test)]
mod tests {
    use crate::{OwnedAudioBuffer, SampleLocation};

    use super::*;

    fn create_sample_with_value(
        num_frames: usize,
        num_channels: usize,
        sample_rate: usize,
        value: f32,
    ) -> OwnedAudioBuffer {
        let mut sample = OwnedAudioBuffer::new(num_frames, num_channels, sample_rate);
        sample.fill_with_value(value);
        sample
    }

    fn process_sampler(
        sampler: &mut SamplerDspProcess,
        num_frames: usize,
        num_channels: usize,
        sample_rate: usize,
    ) -> OwnedAudioBuffer {
        let mut output_buffer = OwnedAudioBuffer::new(num_frames, num_channels, sample_rate);
        let input_buffer = OwnedAudioBuffer::new(num_frames, num_channels, sample_rate);
        let start_time = Timestamp::from_seconds(0.0);

        sampler.process_audio(
            &input_buffer,
            &mut output_buffer,
            &start_time,
            &DspParameterMap::new(),
        );

        output_buffer
    }

    #[test]
    fn fades_in() {
        let num_frames = 10_000;
        let sample_rate = 44_100;
        let num_channels = 1;

        let sample = create_sample_with_value(num_frames, num_channels, sample_rate, 1.0);
        let (mut event_transmitter, event_receiver) = lockfree::channel::spsc::create();
        let mut sampler = SamplerDspProcess::new(sample_rate, sample, event_receiver);

        let _ = event_transmitter.send(SamplerEvent::start(
            Timestamp::from_seconds(0.0),
            Timestamp::from_seconds(0.0),
        ));

        let output_buffer = process_sampler(&mut sampler, num_frames, num_channels, sample_rate);

        approx::assert_relative_eq!(
            0.0,
            output_buffer.get_sample(&SampleLocation {
                frame: 0,
                channel: 0
            })
        );
        approx::assert_relative_eq!(
            0.5,
            output_buffer.get_sample(&SampleLocation {
                frame: sampler.fade.len() / 2,
                channel: 0
            }),
            epsilon = 0.01
        );
        approx::assert_relative_eq!(
            1.0,
            output_buffer.get_sample(&SampleLocation {
                frame: sampler.fade.len(),
                channel: 0
            }),
            epsilon = 0.01
        );
    }

    #[test]
    fn fades_out() {
        let num_frames = 10_000;
        let sample_rate = 44_100;
        let num_channels = 1;

        let sample = create_sample_with_value(num_frames, num_channels, sample_rate, 1.0);
        let (mut event_transmitter, event_receiver) = lockfree::channel::spsc::create();
        let mut sampler = SamplerDspProcess::new(sample_rate, sample, event_receiver);

        let _ = event_transmitter.send(SamplerEvent::start(
            Timestamp::from_seconds(0.0),
            Timestamp::from_seconds(0.0),
        ));

        let fade_length = sampler.fade.len();

        let _ = process_sampler(&mut sampler, 2 * fade_length, num_channels, sample_rate);
        let _ = event_transmitter.send(SamplerEvent::stop(Timestamp::from_seconds(0.0)));
        let output = process_sampler(&mut sampler, 2 * fade_length, num_channels, sample_rate);

        approx::assert_relative_eq!(
            1.0,
            output.get_sample(&SampleLocation {
                frame: 0,
                channel: 0
            })
        );

        approx::assert_relative_eq!(
            0.5,
            output.get_sample(&SampleLocation {
                frame: sampler.fade.len() / 2,
                channel: 0
            }),
            epsilon = 0.01
        );

        approx::assert_relative_eq!(
            0.0,
            output.get_sample(&SampleLocation {
                frame: sampler.fade.len(),
                channel: 0
            }),
            epsilon = 0.01
        );
    }

    #[test]
    fn fade_out_beyond_sample() {
        let num_frames = 10_000;
        let sample_rate = 48_000;
        let num_channels = 2;

        let sample = create_sample_with_value(num_frames, num_channels, sample_rate, 1.0);
        let (mut event_transmitter, event_receiver) = lockfree::channel::spsc::create();
        let mut sampler = SamplerDspProcess::new(sample_rate, sample, event_receiver);

        let _ = event_transmitter.send(SamplerEvent::start(
            Timestamp::from_seconds(0.0),
            Timestamp::from_seconds(0.0),
        ));

        let fade_length = sampler.fade.len();

        let _ = process_sampler(
            &mut sampler,
            num_frames - fade_length / 2,
            num_channels,
            sample_rate,
        );

        let _ = event_transmitter.send(SamplerEvent::stop(Timestamp::from_seconds(0.0)));

        let output = process_sampler(&mut sampler, 2 * fade_length, num_channels, sample_rate);

        approx::assert_relative_eq!(
            1.0,
            output.get_sample(&SampleLocation {
                frame: 0,
                channel: 0
            })
        );

        approx::assert_relative_eq!(
            0.0,
            output.get_sample(&SampleLocation {
                frame: sampler.fade.len(),
                channel: 0
            }),
            epsilon = 0.01
        );
    }
}
