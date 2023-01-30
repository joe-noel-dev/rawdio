use std::time::Duration;

use crate::{
    effects::{utility::EventProcessor, Channel},
    graph::{DspParameters, DspProcessor},
    AudioBuffer, MutableBorrowedAudioBuffer, OwnedAudioBuffer, Timestamp,
};

use super::{
    sampler_event::{SampleEventType, SamplerEvent},
    sampler_fade::Fade,
    sampler_voice::Voice,
};

pub type EventReceiver = Channel::Receiver<SamplerEvent>;
pub type EventTransmitter = Channel::Sender<SamplerEvent>;

pub struct SamplerDspProcess {
    fade: Fade,
    voices: Vec<Voice>,
    active_voice: Option<usize>,
    buffer: OwnedAudioBuffer,
    event_processor: EventProcessor<SamplerEvent>,
    sample_rate: usize,

    loop_points: Option<(Timestamp, Timestamp)>,

    position: Timestamp,
    start_position_in_sample: Timestamp,
    completed_loops: usize,
}

const NUM_VOICES: usize = 2;
const FADE_LENGTH: Duration = Duration::from_millis(50);
const MAX_PENDING_EVENTS: usize = 16;

impl DspProcessor for SamplerDspProcess {
    fn process_audio(
        &mut self,
        _input_buffer: &dyn AudioBuffer,
        output_buffer: &mut dyn AudioBuffer,
        start_time: &Timestamp,
        _parameters: &DspParameters,
    ) {
        debug_assert_eq!(self.sample_rate, output_buffer.sample_rate());

        self.event_processor.process_events();

        let mut current_time = *start_time;
        let mut position = 0;

        while position < output_buffer.frame_count() {
            let (end_frame, event) = self.event_processor.next_event(
                start_time,
                &current_time,
                output_buffer.frame_count(),
            );

            debug_assert!(end_frame <= output_buffer.frame_count());

            let frame_count = end_frame - position;

            let mut slice =
                MutableBorrowedAudioBuffer::slice_frames(output_buffer, position, frame_count);
            self.process_sample(&mut slice);

            position += frame_count;
            current_time = current_time.incremented_by_samples(frame_count, self.sample_rate);

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
            event_processor: EventProcessor::with_capacity(
                MAX_PENDING_EVENTS,
                event_receiver,
                sample_rate,
                |event| event.time,
            ),
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
                return Timestamp::from_samples(self.buffer.frame_count() as f64, self.sample_rate)
            }
        };

        let first_loop_length = loop_end - self.start_position_in_sample;
        let subsequent_loop_length = loop_end - loop_start;

        let sample_position = first_loop_length.as_samples(self.sample_rate)
            + subsequent_loop_length.as_samples(self.sample_rate) * self.completed_loops as f64;

        Timestamp::from_samples(sample_position, self.sample_rate)
    }

    fn get_render_interval(&self, samples_remaining: usize) -> Timestamp {
        let end_of_frame = self
            .position
            .incremented_by_samples(samples_remaining, self.sample_rate);

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

        while frame_position < output_buffer.frame_count() {
            let samples_remaining = output_buffer.frame_count() - frame_position;

            let render_interval = self.get_render_interval(samples_remaining);
            let render_interval = render_interval.as_samples(self.sample_rate).round() as usize;

            let render_frame_count = std::cmp::min(render_interval, samples_remaining);

            if render_frame_count == 0 && self.is_looping() {
                self.finish_sample();
                continue;
            }

            if render_frame_count == 0 && !self.is_looping() {
                break;
            }

            self.process_voices(&mut MutableBorrowedAudioBuffer::slice_frames(
                output_buffer,
                frame_position,
                render_frame_count,
            ));

            frame_position += render_frame_count;

            self.position = self
                .position
                .incremented_by_samples(render_frame_count, self.sample_rate);
        }
    }

    fn process_voices(&mut self, output_buffer: &mut dyn AudioBuffer) {
        let fade = &self.fade;
        let sample = &self.buffer;
        self.voices
            .iter_mut()
            .for_each(|voice| voice.render(output_buffer, sample, fade));
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
            SampleEventType::CancelAll => self.cancel_all(),
        }
    }

    fn set_loop_points(&mut self, loop_start: Timestamp, loop_end: Timestamp) {
        self.loop_points = Some((loop_start, loop_end));
    }

    fn clear_loop_points(&mut self) {
        self.loop_points = None
    }

    fn cancel_all(&mut self) {
        self.event_processor.cancel_all_pending_events();
        self.stop();
    }

    fn assign_voice(&mut self, start_position: Timestamp) {
        let sample_position = start_position.as_samples(self.sample_rate).round() as usize;

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
    use crate::{AudioBuffer, SampleLocation};

    use super::*;

    fn create_sample_with_value(
        frame_count: usize,
        channel_count: usize,
        sample_rate: usize,
        value: f32,
    ) -> OwnedAudioBuffer {
        let mut sample = OwnedAudioBuffer::new(frame_count, channel_count, sample_rate);
        sample.fill_with_value(value);
        sample
    }

    fn process_sampler(
        sampler: &mut SamplerDspProcess,
        frame_count: usize,
        channel_count: usize,
        sample_rate: usize,
    ) -> OwnedAudioBuffer {
        let mut output_buffer = OwnedAudioBuffer::new(frame_count, channel_count, sample_rate);
        let input_buffer = OwnedAudioBuffer::new(frame_count, channel_count, sample_rate);
        let start_time = Timestamp::zero();

        sampler.process_audio(
            &input_buffer,
            &mut output_buffer,
            &start_time,
            &DspParameters::empty(),
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
        let frame_count = 10_000;
        let sample_rate = 44_100;
        let channel_count = 1;

        let sample = create_sample_with_value(frame_count, channel_count, sample_rate, 1.0);
        let (event_transmitter, event_receiver) = crossbeam::channel::unbounded();
        let mut sampler = SamplerDspProcess::new(sample_rate, sample, event_receiver);

        let _ = event_transmitter.send(SamplerEvent::start(
            Timestamp::zero(),
            Timestamp::from_samples(100.0, sample_rate),
        ));

        let output_buffer = process_sampler(&mut sampler, frame_count, channel_count, sample_rate);

        expect_sample(0.0, &output_buffer, 0, 0);
        expect_sample(0.5, &output_buffer, sampler.fade.len() / 2, 0);
        expect_sample(1.0, &output_buffer, sampler.fade.len(), 0);
    }

    #[test]
    fn fades_out() {
        let frame_count = 10_000;
        let sample_rate = 44_100;
        let channel_count = 1;

        let sample = create_sample_with_value(frame_count, channel_count, sample_rate, 1.0);
        let (event_transmitter, event_receiver) = crossbeam::channel::unbounded();
        let mut sampler = SamplerDspProcess::new(sample_rate, sample, event_receiver);

        let _ = event_transmitter.send(SamplerEvent::start_now());
        let _ = event_transmitter.send(SamplerEvent::stop(Timestamp::from_samples(
            5_000.0,
            sample_rate,
        )));

        let output = process_sampler(&mut sampler, 10_000, channel_count, sample_rate);

        expect_sample(1.0, &output, 5_000, 0);
        expect_sample(0.5, &output, 5_000 + sampler.fade.len() / 2, 0);
        expect_sample(0.0, &output, 5_000 + sampler.fade.len(), 0);
    }

    #[test]
    fn fade_out_beyond_sample() {
        let frame_count = 10_000;
        let sample_rate = 48_000;
        let channel_count = 2;

        let sample = create_sample_with_value(frame_count, channel_count, sample_rate, 1.0);
        let (event_transmitter, event_receiver) = crossbeam::channel::unbounded();
        let mut sampler = SamplerDspProcess::new(sample_rate, sample, event_receiver);

        let _ = event_transmitter.send(SamplerEvent::start(Timestamp::zero(), Timestamp::zero()));

        let fade_length = sampler.fade.len();

        let _ = process_sampler(
            &mut sampler,
            frame_count - fade_length / 2,
            channel_count,
            sample_rate,
        );

        let _ = event_transmitter.send(SamplerEvent::stop_now());

        let output = process_sampler(&mut sampler, 2 * fade_length, channel_count, sample_rate);

        expect_sample(1.0, &output, 0, 0);
        expect_sample(0.0, &output, sampler.fade.len(), 0);
    }

    #[test]
    fn start_event() {
        let frame_count = 10_000;
        let sample_rate = 48_000;
        let channel_count = 2;

        let sample = create_sample_with_value(frame_count, channel_count, sample_rate, 1.0);
        let (event_transmitter, event_receiver) = crossbeam::channel::unbounded();
        let mut sampler = SamplerDspProcess::new(sample_rate, sample, event_receiver);

        let start_time_in_samples = 1500;

        let _ = event_transmitter.send(SamplerEvent::start(
            Timestamp::from_samples(start_time_in_samples as f64, sample_rate),
            Timestamp::zero(),
        ));

        let output = process_sampler(&mut sampler, frame_count, channel_count, sample_rate);
        expect_sample(0.0, &output, start_time_in_samples - 1, 0);
        expect_sample(1.0, &output, start_time_in_samples, 0);
    }

    #[test]
    fn stop_event() {
        let frame_count = 10_000;
        let sample_rate = 48_000;
        let channel_count = 2;

        let sample = create_sample_with_value(frame_count, channel_count, sample_rate, 1.0);
        let (event_transmitter, event_receiver) = crossbeam::channel::unbounded();
        let mut sampler = SamplerDspProcess::new(sample_rate, sample, event_receiver);

        let stop_time_in_samples = 2000;

        let _ = event_transmitter.send(SamplerEvent::stop(Timestamp::from_samples(
            stop_time_in_samples as f64,
            sample_rate,
        )));

        let _ = event_transmitter.send(SamplerEvent::start_now());

        let output = process_sampler(&mut sampler, frame_count, channel_count, sample_rate);
        expect_sample(1.0, &output, stop_time_in_samples, 0);
        expect_sample(0.0, &output, stop_time_in_samples + sampler.fade.len(), 0);
    }

    #[test]
    fn loops_samples() {
        let frame_count = 10_000;
        let sample_rate = 48_000;
        let channel_count = 2;

        let mut sample = create_sample_with_value(frame_count, channel_count, sample_rate, 1.0);
        sample.set_sample(SampleLocation::new(0, 4999), 0.4999);

        let (event_transmitter, event_receiver) = crossbeam::channel::unbounded();
        let mut sampler = SamplerDspProcess::new(sample_rate, sample, event_receiver);

        let _ = event_transmitter.send(SamplerEvent::start_now());

        let _ = event_transmitter.send(SamplerEvent::enable_loop(
            Timestamp::from_samples(1_000.0, sample_rate),
            Timestamp::from_samples(5_000.0, sample_rate),
        ));

        let output = process_sampler(&mut sampler, 50_000, channel_count, sample_rate);
        expect_sample(0.4999, &output, 4_999, 0);
        expect_sample(0.4999, &output, 8_999, 0);
        expect_sample(0.4999, &output, 12_999, 0);
    }

    #[test]
    fn loops_and_frame_length_aligned() {
        let frame_count = 10_000;
        let sample_rate = 48_000;
        let channel_count = 2;

        let mut sample = create_sample_with_value(frame_count, channel_count, sample_rate, 1.0);
        sample.set_sample(SampleLocation::new(0, 9999), 0.123);

        let (event_transmitter, event_receiver) = crossbeam::channel::unbounded();
        let mut sampler = SamplerDspProcess::new(sample_rate, sample, event_receiver);

        let _ = event_transmitter.send(SamplerEvent::start_now());

        let _ = event_transmitter.send(SamplerEvent::enable_loop(
            Timestamp::from_samples(0.0, sample_rate),
            Timestamp::from_samples(10_000.0, sample_rate),
        ));

        let output = process_sampler(&mut sampler, 20_000, channel_count, sample_rate);
        expect_sample(0.123, &output, 9_999, 0);
        expect_sample(0.123, &output, 19_999, 0);
        let output = process_sampler(&mut sampler, 20_000, channel_count, sample_rate);
        expect_sample(0.123, &output, 9_999, 0);
        expect_sample(0.123, &output, 19_999, 0);
        let output = process_sampler(&mut sampler, 20_000, channel_count, sample_rate);
        expect_sample(0.123, &output, 9_999, 0);
        expect_sample(0.123, &output, 19_999, 0);
    }

    #[test]
    fn between_sample_looping() {
        let frame_count = 10_000;
        let sample_rate = 48_000;
        let channel_count = 2;

        let sample = create_sample_with_value(frame_count, channel_count, sample_rate, 1.0);

        let (event_transmitter, event_receiver) = crossbeam::channel::unbounded();
        let mut sampler = SamplerDspProcess::new(sample_rate, sample, event_receiver);

        let _ = event_transmitter.send(SamplerEvent::start(
            Timestamp::zero(),
            Timestamp::from_samples(0.5, sample_rate),
        ));

        let _ = event_transmitter.send(SamplerEvent::enable_loop(
            Timestamp::from_samples(0.5, sample_rate),
            Timestamp::from_samples(12.2, sample_rate),
        ));

        let _ = process_sampler(&mut sampler, 1_170, channel_count, sample_rate);
        assert_eq!(99, sampler.completed_loops);
        let _ = process_sampler(&mut sampler, 1, channel_count, sample_rate);
        assert_eq!(100, sampler.completed_loops);
    }
}
