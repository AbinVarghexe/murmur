//! Streaming sample-rate conversion (for example 48 kHz microphone audio down to the
//! 16 kHz the speech model needs).
//!
//! A windowed-sinc interpolator evaluated at each output position. When shrinking the
//! rate the filter cutoff drops with it, so tones above the new Nyquist frequency are
//! removed instead of folding back as noise. Chunks can be pushed as they arrive.

use std::f64::consts::PI;

/// Half-width of the interpolation kernel, in output-rate samples.
const KERNEL_RADIUS: f64 = 16.0;

#[derive(Debug, Clone)]
pub struct Resampler {
    /// Input samples advanced per output sample.
    step: f64,
    /// Filter cutoff as a fraction of the input Nyquist frequency.
    cutoff: f64,
    half_width: usize,
    buffer: Vec<f32>,
    /// Position of the next output sample, in `buffer` coordinates.
    next_pos: f64,
}

impl Resampler {
    pub fn new(in_rate: u32, out_rate: u32) -> Self {
        let step = f64::from(in_rate) / f64::from(out_rate.max(1));
        let cutoff = (1.0 / step).min(1.0);
        let half_width = (KERNEL_RADIUS / cutoff).ceil() as usize;
        Self {
            step,
            cutoff,
            half_width,
            buffer: Vec::new(),
            next_pos: 0.0,
        }
    }

    fn kernel(&self, distance: f64) -> f64 {
        let radius = self.half_width as f64;
        if distance.abs() >= radius {
            return 0.0;
        }
        let x = distance * self.cutoff;
        let sinc = if x.abs() < 1e-9 {
            1.0
        } else {
            (PI * x).sin() / (PI * x)
        };
        let window = 0.5 * (1.0 + (PI * distance / radius).cos());
        self.cutoff * sinc * window
    }

    /// Convert a chunk and append the result to `out`.
    pub fn push(&mut self, input: &[f32], out: &mut Vec<f32>) {
        self.buffer.extend_from_slice(input);
        loop {
            let center = self.next_pos;
            let base = center.floor() as isize;
            let last_needed = base + self.half_width as isize;
            if last_needed as usize >= self.buffer.len() {
                break;
            }
            let mut sum = 0.0_f64;
            let mut weights = 0.0_f64;
            let first = base - self.half_width as isize + 1;
            for index in first..=last_needed {
                let weight = self.kernel(center - index as f64);
                weights += weight;
                if index >= 0 {
                    sum += f64::from(self.buffer[index as usize]) * weight;
                }
            }
            out.push(if weights.abs() > 1e-12 {
                (sum / weights) as f32
            } else {
                0.0
            });
            self.next_pos += self.step;
        }
        // Drop input that no later output can reach.
        let keep_from = (self.next_pos.floor() as isize - self.half_width as isize).max(0) as usize;
        if keep_from > 0 {
            self.buffer.drain(..keep_from);
            self.next_pos -= keep_from as f64;
        }
    }

    /// Convenience: convert a whole recording at once.
    pub fn convert(in_rate: u32, out_rate: u32, input: &[f32]) -> Vec<f32> {
        let mut resampler = Resampler::new(in_rate, out_rate);
        let mut out =
            Vec::with_capacity(input.len() * out_rate as usize / in_rate.max(1) as usize + 1);
        resampler.push(input, &mut out);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio_utils::rms;

    fn sine(rate: u32, hz: f32, seconds: f32, amplitude: f32) -> Vec<f32> {
        (0..(rate as f32 * seconds) as usize)
            .map(|n| amplitude * (2.0 * std::f32::consts::PI * hz * n as f32 / rate as f32).sin())
            .collect()
    }

    #[test]
    fn same_rate_is_an_identity() {
        let input = sine(16_000, 440.0, 0.5, 0.5);
        let output = Resampler::convert(16_000, 16_000, &input);
        assert!(output.len() >= input.len() - 40);
        for (a, b) in input.iter().zip(&output).skip(40).take(1000) {
            assert!((a - b).abs() < 1e-3);
        }
    }

    #[test]
    fn downsampling_48k_to_16k_keeps_a_voice_range_tone() {
        let input = sine(48_000, 1_000.0, 1.0, 0.5);
        let output = Resampler::convert(48_000, 16_000, &input);
        assert!(
            (output.len() as i64 - 16_000).abs() < 80,
            "length {}",
            output.len()
        );
        let level = rms(&output[400..output.len() - 400]);
        assert!((level - 0.5 / 2.0_f32.sqrt()).abs() < 0.01, "rms {level}");
    }

    #[test]
    fn downsampling_44k1_to_16k_has_the_right_length() {
        let input = sine(44_100, 500.0, 1.0, 0.4);
        let output = Resampler::convert(44_100, 16_000, &input);
        assert!(
            (output.len() as i64 - 16_000).abs() < 80,
            "length {}",
            output.len()
        );
    }

    #[test]
    fn removes_tones_above_the_new_nyquist_frequency() {
        let input = sine(48_000, 12_000.0, 1.0, 0.5);
        let output = Resampler::convert(48_000, 16_000, &input);
        assert!(rms(&output[400..output.len() - 400]) < 0.02);
    }

    #[test]
    fn chunked_input_matches_a_single_pass() {
        let input = sine(48_000, 700.0, 0.6, 0.5);
        let whole = Resampler::convert(48_000, 16_000, &input);

        let mut resampler = Resampler::new(48_000, 16_000);
        let mut chunked = Vec::new();
        for chunk in input.chunks(333) {
            resampler.push(chunk, &mut chunked);
        }
        assert_eq!(whole.len(), chunked.len());
        for (a, b) in whole.iter().zip(&chunked) {
            assert!((a - b).abs() < 1e-5);
        }
    }

    #[test]
    fn empty_input_gives_empty_output() {
        assert!(Resampler::convert(48_000, 16_000, &[]).is_empty());
    }
}
