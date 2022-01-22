use crate::{AudioBuffer, SampleLocation};

use super::fade::Fade;

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

        while destination_offset < output.num_frames() {
            match self.phase {
                Phase::Stopped => break,
                Phase::FadingIn(fade_position) => {
                    let num_frames = self.render_fade(
                        output,
                        destination_offset,
                        sample,
                        fade,
                        fade_position,
                        true,
                    );

                    self.position += num_frames;
                    destination_offset += num_frames;

                    let fade_position = fade_position + num_frames;

                    if fade_position < fade.len() {
                        self.phase = Phase::FadingIn(fade_position);
                    } else {
                        self.phase = Phase::Playing;
                    }
                }
                Phase::Playing => {
                    self.render_playing(output, destination_offset, sample);

                    self.position += output.num_frames() - destination_offset;
                    destination_offset = output.num_frames();
                }
                Phase::FadingOut(fade_position) => {
                    let num_frames = self.render_fade(
                        output,
                        destination_offset,
                        sample,
                        fade,
                        fade_position,
                        false,
                    );

                    self.position += num_frames;
                    destination_offset += num_frames;

                    let fade_position = fade_position + num_frames;
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
        let num_channels = min(source.num_channels(), output.num_channels());

        if self.position >= source.num_frames() {
            return;
        }

        let num_frames = std::cmp::min(
            output.num_frames() - destination_offset,
            source.num_frames() - self.position,
        );

        let source_location = SampleLocation::new(0, self.position);
        let destination_location = SampleLocation::new(0, destination_offset);

        output.add_from(
            source,
            &source_location,
            &destination_location,
            num_channels,
            num_frames,
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
        let num_channels = min(source.num_channels(), output.num_channels());

        let num_frames = std::cmp::min(
            fade.len() - fade_position,
            output.num_frames() - destination_offset,
        );

        for frame in 0..num_frames {
            if frame + self.position >= source.num_frames() {
                break;
            }

            let fade_value = if fade_in {
                fade.fade_in_value(fade_position + frame)
            } else {
                fade.fade_out_value(fade_position + frame)
            };

            for channel in 0..num_channels {
                let source_location = SampleLocation {
                    channel,
                    frame: self.position + frame,
                };

                let dest_location = SampleLocation {
                    channel,
                    frame: destination_offset + frame,
                };

                let sample = output.get_sample(&dest_location)
                    + fade_value * source.get_sample(&source_location);
                output.set_sample(&dest_location, sample);
            }
        }

        num_frames
    }
}
