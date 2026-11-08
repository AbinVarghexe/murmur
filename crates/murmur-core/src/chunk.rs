//! Splits long audio so the speech model never sees more than it handles well, cutting
//! at the quietest point near the limit so words are not sliced in half.

const FRAME_SECONDS: f32 = 0.02;

fn frame_energy(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    samples.iter().map(|v| v * v).sum::<f32>() / samples.len() as f32
}

fn quietest_cut(samples: &[f32], search_from: usize, end: usize, frame: usize) -> usize {
    let mut best_cut = end;
    let mut best_energy = f32::INFINITY;
    let mut pos = search_from;
    while pos + frame <= end {
        let energy = frame_energy(&samples[pos..pos + frame]);
        if energy < best_energy {
            best_energy = energy;
            best_cut = pos + frame / 2;
        }
        pos += frame;
    }
    best_cut
}

/// Split into chunks no longer than `max_seconds`. Chunks borrow from the input.
pub fn split_at_silence(
    samples: &[f32],
    sample_rate: u32,
    max_seconds: f32,
    search_seconds: f32,
) -> Vec<&[f32]> {
    let max = (max_seconds * sample_rate as f32) as usize;
    let search = (search_seconds * sample_rate as f32) as usize;
    let frame = ((FRAME_SECONDS * sample_rate as f32) as usize).max(1);
    if max == 0 {
        return vec![samples];
    }

    let mut chunks = Vec::new();
    let mut start = 0;
    while samples.len() - start > max {
        let end = start + max;
        let search_from = (start + frame).max(end.saturating_sub(search));
        let cut = quietest_cut(samples, search_from, end, frame);
        let safe_cut = if cut > start { cut } else { end };
        chunks.push(&samples[start..safe_cut]);
        start = safe_cut;
    }
    if start < samples.len() {
        chunks.push(&samples[start..]);
    }
    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    const RATE: u32 = 1000;

    fn loud(seconds: usize) -> Vec<f32> {
        vec![0.5; seconds * RATE as usize]
    }

    #[test]
    fn returns_a_single_chunk_for_short_audio() {
        assert_eq!(split_at_silence(&loud(10), RATE, 30.0, 10.0).len(), 1);
    }

    #[test]
    fn cuts_at_the_quiet_gap_and_never_exceeds_the_maximum() {
        let mut audio = loud(70);
        for v in &mut audio[25_000..25_200] {
            *v = 0.0;
        }
        let chunks = split_at_silence(&audio, RATE, 30.0, 10.0);
        assert!(chunks.len() > 1);
        let first_cut = chunks[0].len();
        assert!((25_000..=25_200).contains(&first_cut));
        assert!(chunks.iter().all(|c| c.len() <= 30_000));
    }

    #[test]
    fn preserves_every_sample() {
        let audio = loud(95);
        let total: usize = split_at_silence(&audio, RATE, 30.0, 10.0)
            .iter()
            .map(|c| c.len())
            .sum();
        assert_eq!(total, audio.len());
    }

    #[test]
    fn handles_empty_input() {
        assert!(split_at_silence(&[], RATE, 30.0, 10.0).is_empty());
    }
}
