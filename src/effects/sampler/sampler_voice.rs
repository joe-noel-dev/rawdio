use crate::{AudioBuffer, SampleLocation};

use super::sampler_fade::Fade;

use std::cmp::min;

#[derive(PartialEq)]
enum Phase {
    Stopped,
    FadingIn(usize),
    Playing,
    FadingOut(usize),
}

impl Default for Phase {
    fn default() -> Self {
        Self::Stopped
    }
}

#[derive(Default)]
pub struct Voice {
    position: usize,
    phase: Phase,
}

impl Voice {
    pub fn is_stopped(&self) -> bool {
        self.phase == Phase::Stopped
    }

    pub fn start_from_position(&mut self, position: usize) {
        self.position = position;
        self.phase = match position {
            0 => Phase::Playing,
            _ => Phase::FadingIn(0),
        };
    }

    pub fn get_position(&self) -> usize {
        self.position
    }

    pub fn stop(&mut self) {
        match self.phase {
            Phase::Stopped => (),
            Phase::FadingIn(_) => self.phase = Phase::FadingOut(0),
            Phase::Playing => self.phase = Phase::FadingOut(0),
            Phase::FadingOut(_) => (),
        }
    }

    pub fn render(&mut self, output: &mut dyn AudioBuffer, sample: &dyn AudioBuffer, fade: &Fade) {
        if self.is_stopped() {
            return;
        }

        let mut destination_offset = 0;

        while destination_offset < output.frame_count() {
            match self.phase {
                Phase::Stopped => break,
                Phase::FadingIn(fade_position) => {
                    let frame_count = self.render_fade(
                        output,
                        destination_offset,
                        sample,
                        fade,
                        fade_position,
                        true,
                    );

                    self.position += frame_count;
                    destination_offset += frame_count;

                    let fade_position = fade_position + frame_count;

                    if fade_position < fade.len() {
                        self.phase = Phase::FadingIn(fade_position);
                    } else {
                        self.phase = Phase::Playing;
                    }
                }
                Phase::Playing => {
                    self.render_playing(output, destination_offset, sample);

                    self.position += output.frame_count() - destination_offset;
                    destination_offset = output.frame_count();
                }
                Phase::FadingOut(fade_position) => {
                    let frame_count = self.render_fade(
                        output,
                        destination_offset,
                        sample,
                        fade,
                        fade_position,
                        false,
                    );

                    self.position += frame_count;
                    destination_offset += frame_count;

                    let fade_position = fade_position + frame_count;
                    if fade_position < fade.len() {
                        self.phase = Phase::FadingOut(fade_position);
                    } else {
                        self.phase = Phase::Stopped;
                    }
                }
            };
        }
    }

    pub fn render_playing(
        &mut self,
        output: &mut dyn AudioBuffer,
        destination_offset: usize,
        source: &dyn AudioBuffer,
    ) {
        let channel_count = min(source.channel_count(), output.channel_count());

        if self.position >= source.frame_count() {
            return;
        }

        let frame_count = std::cmp::min(
            output.frame_count() - destination_offset,
            source.frame_count() - self.position,
        );

        let source_location = SampleLocation::frame(self.position);
        let destination_location = SampleLocation::frame(destination_offset);

        output.add_from(
            source,
            source_location,
            destination_location,
            channel_count,
            frame_count,
        );
    }

    pub fn render_fade(
        &mut self,
        output: &mut dyn AudioBuffer,
        destination_offset: usize,
        source: &dyn AudioBuffer,
        fade: &Fade,
        fade_position: usize,
        fade_in: bool,
    ) -> usize {
        let channel_count = min(source.channel_count(), output.channel_count());

        let frame_count = std::cmp::min(
            fade.len() - fade_position,
            output.frame_count() - destination_offset,
        );

        for frame in 0..frame_count {
            if frame + self.position >= source.frame_count() {
                break;
            }

            let fade_value = if fade_in {
                fade.fade_in_value(fade_position + frame)
            } else {
                fade.fade_out_value(fade_position + frame)
            };

            for channel in 0..channel_count {
                let source_location = SampleLocation {
                    channel,
                    frame: self.position + frame,
                };

                let dest_location = SampleLocation {
                    channel,
                    frame: destination_offset + frame,
                };

                let sample = output.get_sample(dest_location)
                    + fade_value * source.get_sample(source_location);
                output.set_sample(dest_location, sample);
            }
        }

        frame_count
    }
}
