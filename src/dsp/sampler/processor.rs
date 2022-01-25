use std::time::Duration;

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
    sample_rate: usize,

    loop_points: Option<(Timestamp, Timestamp)>,

    position: Timestamp,
    start_position_in_sample: Timestamp,
    completed_loops: usize,
}

const NUM_VOICES: usize = 2;
const FADE_LENGTH: Duration = Duration::from_millis(50);
const MAX_PENDING_EVENTS: usize = 10;

pub enum SampleEventType {
    Start(Timestamp),
    Stop,

    EnableLoop(Timestamp, Timestamp),
    CancelLoop,
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

    pub fn start_now() -> Self {
        Self {
            time: Timestamp::zero(),
            event_type: SampleEventType::Start(Timestamp::zero()),
        }
    }

    pub fn stop(stop_at_time: Timestamp) -> Self {
        Self {
            time: stop_at_time,
            event_type: SampleEventType::Stop,
        }
    }

    pub fn stop_now() -> Self {
        Self {
            time: Timestamp::zero(),
            event_type: SampleEventType::Stop,
        }
    }

    pub fn enable_loop(loop_start: Timestamp, loop_end: Timestamp) -> Self {
        Self {
            time: Timestamp::zero(),
            event_type: SampleEventType::EnableLoop(loop_start, loop_end),
        }
    }

    pub fn cancel_loop() -> Self {
        Self {
            time: Timestamp::zero(),
            event_type: SampleEventType::CancelLoop,
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
        debug_assert_eq!(self.sample_rate, output_buffer.sample_rate());

        self.read_events();

        let mut current_time = *start_time;
        let mut position = 0;

        while position < output_buffer.num_frames() {
            let (end_frame, event) =
                self.next_event_position(start_time, &current_time, output_buffer.num_frames());

            debug_assert!(end_frame <= output_buffer.num_frames());
            let num_frames = end_frame - position;

            self.process_sample(&mut AudioBufferSlice::new(
                output_buffer,
                position,
                num_frames,
            ));

            position += num_frames;
            current_time = current_time.incremented_by_samples(num_frames, self.sample_rate);

            if let Some(event) = event {
                self.process_event(&event);
            }
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
            fade: Fade::new(FADE_LENGTH, sample_rate),
            voices: (0..NUM_VOICES).map(|_| Voice::default()).collect(),
            active_voice: None,
            buffer,
            event_receiver,
            pending_events: Vec::with_capacity(MAX_PENDING_EVENTS),
            loop_points: None,
            position: Timestamp::zero(),
            start_position_in_sample: Timestamp::zero(),
            completed_loops: 0,
            sample_rate,
        }
    }

    fn next_loop_position(&self) -> Timestamp {
        let (loop_start, loop_end) = match self.loop_points {
            Some(loop_points) => loop_points,
            None => {
                return Timestamp::from_samples(self.buffer.num_frames() as f64, self.sample_rate)
            }
        };

        let first_loop_length = loop_end - self.start_position_in_sample;
        let subsequent_loop_length = loop_end - loop_start;

        let sample_position = first_loop_length.get_samples(self.sample_rate)
            + subsequent_loop_length.get_samples(self.sample_rate) * self.completed_loops as f64;

        Timestamp::from_samples(sample_position, self.sample_rate)
    }

    fn get_render_interval(&self, num_samples_remaining_in_frame: usize) -> Timestamp {
        let end_of_frame = self
            .position
            .incremented_by_samples(num_samples_remaining_in_frame, self.sample_rate);

        let end_of_sample = self.next_loop_position();

        let end_point = if end_of_frame < end_of_sample {
            end_of_frame
        } else {
            end_of_sample
        };

        end_point - self.position
    }

    fn finish_sample(&mut self) {
        if let Some((loop_start, _)) = self.loop_points {
            self.loop_back(loop_start);
        }
    }

    fn is_looping(&self) -> bool {
        self.loop_points.is_some()
    }

    fn process_sample(&mut self, output_buffer: &mut dyn AudioBuffer) {
        let mut frame_position = 0;

        while frame_position < output_buffer.num_frames() {
            let num_samples_remaining_in_frame = output_buffer.num_frames() - frame_position;

            let render_interval = self.get_render_interval(num_samples_remaining_in_frame);
            let render_interval = render_interval.get_samples(self.sample_rate).round() as usize;

            let num_frames_to_render =
                std::cmp::min(render_interval, num_samples_remaining_in_frame);

            if num_frames_to_render == 0 && self.is_looping() {
                self.finish_sample();
                continue;
            }

            if num_frames_to_render == 0 && !self.is_looping() {
                break;
            }

            self.process_voices(&mut AudioBufferSlice::new(
                output_buffer,
                frame_position,
                num_frames_to_render,
            ));

            frame_position += num_frames_to_render;

            self.position = self
                .position
                .incremented_by_samples(num_frames_to_render, self.sample_rate);
        }
    }

    fn process_voices(&mut self, output_buffer: &mut dyn AudioBuffer) {
        let fade = &self.fade;
        let sample = &self.buffer;
        self.voices
            .iter_mut()
            .for_each(|voice| voice.render(output_buffer, sample, fade));
    }

    fn next_event_position(
        &mut self,
        frame_start_time: &Timestamp,
        current_frame_position: &Timestamp,
        number_of_frames: usize,
    ) -> (usize, Option<SamplerEvent>) {
        let frame_end_time =
            frame_start_time.incremented_by_samples(number_of_frames, self.sample_rate);

        if let Some(next_event) = self.next_event_before(&frame_end_time) {
            let event_time = std::cmp::max(next_event.time, *current_frame_position);
            let position_in_frame = event_time - *frame_start_time;
            (
                position_in_frame.get_samples(self.sample_rate).floor() as usize,
                Some(next_event),
            )
        } else {
            (number_of_frames, None)
        }
    }

    fn next_event_before(&mut self, end_time: &Timestamp) -> Option<SamplerEvent> {
        if let Some(next_event) = self.pending_events.first() {
            if next_event.time < *end_time {
                return Some(self.pending_events.remove(0));
            }
        }

        None
    }

    fn process_event(&mut self, event: &SamplerEvent) {
        match event.event_type {
            SampleEventType::Start(position_in_sample) => {
                self.start(position_in_sample);
            }
            SampleEventType::Stop => self.stop(),
            SampleEventType::EnableLoop(loop_start, loop_end) => {
                self.set_loop_points(loop_start, loop_end)
            }
            SampleEventType::CancelLoop => self.clear_loop_points(),
        }
    }

    fn set_loop_points(&mut self, loop_start: Timestamp, loop_end: Timestamp) {
        self.loop_points = Some((loop_start, loop_end));
    }

    fn clear_loop_points(&mut self) {
        self.loop_points = None
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

    fn assign_voice(&mut self, start_position: Timestamp) {
        let sample_position = start_position.get_samples(self.sample_rate).round() as usize;

        if let Some(current_position) = self.get_active_voice_position() {
            if current_position == sample_position {
                return;
            }
        }

        self.stop();

        if let Some((index, free_voice)) = self
            .voices
            .iter_mut()
            .enumerate()
            .find(|(_, voice)| voice.is_stopped())
        {
            free_voice.start_from_position(sample_position);
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

    fn start(&mut self, from_position: Timestamp) {
        self.assign_voice(from_position);
        self.completed_loops = 0;
        self.position = Timestamp::zero();
        self.start_position_in_sample = from_position;
    }

    fn loop_back(&mut self, from_position: Timestamp) {
        self.completed_loops += 1;
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
        let start_time = Timestamp::zero();

        sampler.process_audio(
            &input_buffer,
            &mut output_buffer,
            &start_time,
            &DspParameterMap::new(),
        );

        output_buffer
    }

    fn expect_sample(expected_value: f32, buffer: &dyn AudioBuffer, frame: usize, channel: usize) {
        approx::assert_relative_eq!(
            expected_value,
            buffer.get_sample(SampleLocation::new(channel, frame)),
            epsilon = 1e-2
        );
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
            Timestamp::zero(),
            Timestamp::from_samples(100.0, sample_rate),
        ));

        let output_buffer = process_sampler(&mut sampler, num_frames, num_channels, sample_rate);

        expect_sample(0.0, &output_buffer, 0, 0);
        expect_sample(0.5, &output_buffer, sampler.fade.len() / 2, 0);
        expect_sample(1.0, &output_buffer, sampler.fade.len(), 0);
    }

    #[test]
    fn fades_out() {
        let num_frames = 10_000;
        let sample_rate = 44_100;
        let num_channels = 1;

        let sample = create_sample_with_value(num_frames, num_channels, sample_rate, 1.0);
        let (mut event_transmitter, event_receiver) = lockfree::channel::spsc::create();
        let mut sampler = SamplerDspProcess::new(sample_rate, sample, event_receiver);

        let _ = event_transmitter.send(SamplerEvent::start_now());
        let _ = event_transmitter.send(SamplerEvent::stop(Timestamp::from_samples(
            5_000.0,
            sample_rate,
        )));

        let output = process_sampler(&mut sampler, 10_000, num_channels, sample_rate);

        expect_sample(1.0, &output, 5_000, 0);
        expect_sample(0.5, &output, 5_000 + sampler.fade.len() / 2, 0);
        expect_sample(0.0, &output, 5_000 + sampler.fade.len(), 0);
    }

    #[test]
    fn fade_out_beyond_sample() {
        let num_frames = 10_000;
        let sample_rate = 48_000;
        let num_channels = 2;

        let sample = create_sample_with_value(num_frames, num_channels, sample_rate, 1.0);
        let (mut event_transmitter, event_receiver) = lockfree::channel::spsc::create();
        let mut sampler = SamplerDspProcess::new(sample_rate, sample, event_receiver);

        let _ = event_transmitter.send(SamplerEvent::start(Timestamp::zero(), Timestamp::zero()));

        let fade_length = sampler.fade.len();

        let _ = process_sampler(
            &mut sampler,
            num_frames - fade_length / 2,
            num_channels,
            sample_rate,
        );

        let _ = event_transmitter.send(SamplerEvent::stop_now());

        let output = process_sampler(&mut sampler, 2 * fade_length, num_channels, sample_rate);

        expect_sample(1.0, &output, 0, 0);
        expect_sample(0.0, &output, sampler.fade.len(), 0);
    }

    #[test]
    fn start_event() {
        let num_frames = 10_000;
        let sample_rate = 48_000;
        let num_channels = 2;

        let sample = create_sample_with_value(num_frames, num_channels, sample_rate, 1.0);
        let (mut event_transmitter, event_receiver) = lockfree::channel::spsc::create();
        let mut sampler = SamplerDspProcess::new(sample_rate, sample, event_receiver);

        let start_time_in_samples = 1500;

        let _ = event_transmitter.send(SamplerEvent::start(
            Timestamp::from_samples(start_time_in_samples as f64, sample_rate),
            Timestamp::zero(),
        ));

        let output = process_sampler(&mut sampler, num_frames, num_channels, sample_rate);
        expect_sample(0.0, &output, start_time_in_samples - 1, 0);
        expect_sample(1.0, &output, start_time_in_samples, 0);
    }

    #[test]
    fn stop_event() {
        let num_frames = 10_000;
        let sample_rate = 48_000;
        let num_channels = 2;

        let sample = create_sample_with_value(num_frames, num_channels, sample_rate, 1.0);
        let (mut event_transmitter, event_receiver) = lockfree::channel::spsc::create();
        let mut sampler = SamplerDspProcess::new(sample_rate, sample, event_receiver);

        let stop_time_in_samples = 2000;

        let _ = event_transmitter.send(SamplerEvent::stop(Timestamp::from_samples(
            stop_time_in_samples as f64,
            sample_rate,
        )));

        let _ = event_transmitter.send(SamplerEvent::start_now());

        let output = process_sampler(&mut sampler, num_frames, num_channels, sample_rate);
        expect_sample(1.0, &output, stop_time_in_samples, 0);
        expect_sample(0.0, &output, stop_time_in_samples + sampler.fade.len(), 0);
    }

    #[test]
    fn loops_samples() {
        let num_frames = 10_000;
        let sample_rate = 48_000;
        let num_channels = 2;

        let mut sample = create_sample_with_value(num_frames, num_channels, sample_rate, 1.0);
        sample.set_sample(SampleLocation::new(0, 4999), 0.4999);

        let (mut event_transmitter, event_receiver) = lockfree::channel::spsc::create();
        let mut sampler = SamplerDspProcess::new(sample_rate, sample, event_receiver);

        let _ = event_transmitter.send(SamplerEvent::start_now());

        let _ = event_transmitter.send(SamplerEvent::enable_loop(
            Timestamp::from_samples(1_000.0, sample_rate),
            Timestamp::from_samples(5_000.0, sample_rate),
        ));

        let output = process_sampler(&mut sampler, 50_000, num_channels, sample_rate);
        expect_sample(0.4999, &output, 4_999, 0);
        expect_sample(0.4999, &output, 8_999, 0);
        expect_sample(0.4999, &output, 12_999, 0);
    }

    #[test]
    fn loops_and_frame_length_aligned() {
        let num_frames = 10_000;
        let sample_rate = 48_000;
        let num_channels = 2;

        let mut sample = create_sample_with_value(num_frames, num_channels, sample_rate, 1.0);
        sample.set_sample(SampleLocation::new(0, 9999), 0.123);

        let (mut event_transmitter, event_receiver) = lockfree::channel::spsc::create();
        let mut sampler = SamplerDspProcess::new(sample_rate, sample, event_receiver);

        let _ = event_transmitter.send(SamplerEvent::start_now());

        let _ = event_transmitter.send(SamplerEvent::enable_loop(
            Timestamp::from_samples(0.0, sample_rate),
            Timestamp::from_samples(10_000.0, sample_rate),
        ));

        let output = process_sampler(&mut sampler, 20_000, num_channels, sample_rate);
        expect_sample(0.123, &output, 9_999, 0);
        expect_sample(0.123, &output, 19_999, 0);
        let output = process_sampler(&mut sampler, 20_000, num_channels, sample_rate);
        expect_sample(0.123, &output, 9_999, 0);
        expect_sample(0.123, &output, 19_999, 0);
        let output = process_sampler(&mut sampler, 20_000, num_channels, sample_rate);
        expect_sample(0.123, &output, 9_999, 0);
        expect_sample(0.123, &output, 19_999, 0);
    }

    #[test]
    fn between_sample_looping() {
        let num_frames = 10_000;
        let sample_rate = 48_000;
        let num_channels = 2;

        let sample = create_sample_with_value(num_frames, num_channels, sample_rate, 1.0);

        let (mut event_transmitter, event_receiver) = lockfree::channel::spsc::create();
        let mut sampler = SamplerDspProcess::new(sample_rate, sample, event_receiver);

        let _ = event_transmitter.send(SamplerEvent::start(
            Timestamp::zero(),
            Timestamp::from_samples(0.5, sample_rate),
        ));

        let _ = event_transmitter.send(SamplerEvent::enable_loop(
            Timestamp::from_samples(0.5, sample_rate),
            Timestamp::from_samples(12.2, sample_rate),
        ));

        let _ = process_sampler(&mut sampler, 1_170, num_channels, sample_rate);
        assert_eq!(99, sampler.completed_loops);
        let _ = process_sampler(&mut sampler, 1, num_channels, sample_rate);
        assert_eq!(100, sampler.completed_loops);
    }
}
