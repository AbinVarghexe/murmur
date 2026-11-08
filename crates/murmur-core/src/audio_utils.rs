//! Small, allocation-light helpers for preparing audio for the speech model.

/// Root-mean-square level of a block of samples, 0.0 to 1.0.
pub fn rms(block: &[f32]) -> f32 {
    if block.is_empty() {
        return 0.0;
    }
    (block.iter().map(|v| v * v).sum::<f32>() / block.len() as f32).sqrt()
}

/// Mix interleaved multi-channel audio down to mono by averaging channels.
pub fn to_mono(interleaved: &[f32], channels: usize) -> Vec<f32> {
    if channels <= 1 {
        return interleaved.to_vec();
    }
    interleaved
        .chunks_exact(channels)
        .map(|frame| frame.iter().sum::<f32>() / channels as f32)
        .collect()
}

/// Add silence before and after. Transducer models drop the first and last word less
/// often when the speech is not flush against the edges of the buffer.
pub fn pad_silence(samples: &[f32], sample_rate: u32, seconds: f32) -> Vec<f32> {
    let pad = (sample_rate as f32 * seconds) as usize;
    let mut padded = Vec::with_capacity(samples.len() + 2 * pad);
    padded.resize(pad, 0.0);
    padded.extend_from_slice(samples);
    padded.resize(padded.len() + pad, 0.0);
    padded
}

/// Raise the volume of a quiet recording. Normal-level audio is left exactly as it is,
/// because aggressive normalising was measured to change results for the worse.
pub fn lift_if_quiet(
    samples: &[f32],
    quiet_peak: f32,
    target_peak: f32,
    max_gain: f32,
) -> Vec<f32> {
    let peak = samples.iter().fold(0.0_f32, |acc, v| acc.max(v.abs()));
    if peak == 0.0 || peak >= quiet_peak {
        return samples.to_vec();
    }
    let gain = (target_peak / peak).min(max_gain);
    samples
        .iter()
        .map(|v| (v * gain).clamp(-1.0, 1.0))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rms_of_silence_and_a_constant_signal() {
        assert_eq!(rms(&[]), 0.0);
        assert_eq!(rms(&[0.0; 10]), 0.0);
        assert!((rms(&[0.5; 100]) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn mixes_stereo_to_mono() {
        assert_eq!(to_mono(&[1.0, 0.0, 0.5, 0.5], 2), vec![0.5, 0.5]);
        assert_eq!(to_mono(&[0.1, 0.2], 1), vec![0.1, 0.2]);
    }

    #[test]
    fn pads_both_ends_with_silence() {
        let padded = pad_silence(&[1.0, 1.0], 10, 0.3);
        assert_eq!(padded.len(), 2 + 2 * 3);
        assert_eq!(&padded[..3], &[0.0, 0.0, 0.0]);
        assert_eq!(&padded[3..5], &[1.0, 1.0]);
        assert_eq!(&padded[5..], &[0.0, 0.0, 0.0]);
    }

    #[test]
    fn leaves_normal_audio_untouched() {
        let loud = vec![0.4, -0.5, 0.3];
        assert_eq!(lift_if_quiet(&loud, 0.25, 0.5, 10.0), loud);
    }

    #[test]
    fn lifts_quiet_audio_but_caps_the_gain() {
        let quiet = vec![0.02, -0.04];
        let lifted = lift_if_quiet(&quiet, 0.25, 0.5, 10.0);
        assert!((lifted[1] + 0.4).abs() < 1e-6); // gain capped at 10x
        let very_quiet = vec![0.001];
        assert!((lift_if_quiet(&very_quiet, 0.25, 0.5, 10.0)[0] - 0.01).abs() < 1e-6);
    }

    #[test]
    fn silence_stays_silence() {
        assert_eq!(lift_if_quiet(&[0.0, 0.0], 0.25, 0.5, 10.0), vec![0.0, 0.0]);
    }
}
