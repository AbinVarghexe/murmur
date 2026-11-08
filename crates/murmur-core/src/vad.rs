//! Decides when a recording should end on its own.
//!
//! It listens to the loudness of each audio block. Once real speech has been heard, a
//! long enough quiet stretch ends the recording. If nothing is said for a while, it
//! gives up. Time is counted from the audio itself, not the wall clock, so behaviour is
//! exact and testable.

/// Never treat anything quieter than this as speech, however quiet the room is.
const MIN_SPEECH_LEVEL: f32 = 0.012;
/// Speech must be this many times louder than the measured background level.
const NOISE_MARGIN: f32 = 2.5;
/// Loud audio shorter than this (a click or cough) is not speech.
const MIN_SPEECH_MS: f32 = 120.0;
/// How quickly the background estimate follows quiet audio.
const FLOOR_ADAPT: f32 = 0.02;
const INITIAL_FLOOR: f32 = 0.004;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Continue,
    /// Speech was heard and then the quiet window passed.
    Silence,
    /// No speech at all within the allowed time.
    NoSpeech,
}

#[derive(Debug, Clone)]
pub struct SilenceDetector {
    silence_ms: f32,
    no_speech_ms: f32,
    elapsed_ms: f32,
    floor: f32,
    speech_run_ms: f32,
    speech_seen: bool,
    last_speech_ms: f32,
}

impl SilenceDetector {
    pub fn new(silence_ms: u32, no_speech_ms: u32) -> Self {
        Self {
            silence_ms: silence_ms as f32,
            no_speech_ms: no_speech_ms as f32,
            elapsed_ms: 0.0,
            floor: INITIAL_FLOOR,
            speech_run_ms: 0.0,
            speech_seen: false,
            last_speech_ms: 0.0,
        }
    }

    /// Feed one audio block: its RMS level (0..1) and its length in milliseconds.
    pub fn push(&mut self, rms: f32, block_ms: f32) -> Verdict {
        self.elapsed_ms += block_ms;
        let threshold = MIN_SPEECH_LEVEL.max(self.floor * NOISE_MARGIN);

        if rms >= threshold {
            self.speech_run_ms += block_ms;
            if self.speech_run_ms >= MIN_SPEECH_MS {
                self.speech_seen = true;
            }
            if self.speech_seen {
                self.last_speech_ms = self.elapsed_ms;
            }
        } else {
            // Quiet audio: let the speech run fade (words have gaps) and learn the room.
            self.speech_run_ms = (self.speech_run_ms - block_ms).max(0.0);
            self.floor = self.floor * (1.0 - FLOOR_ADAPT) + rms * FLOOR_ADAPT;
        }

        if self.speech_seen {
            if self.elapsed_ms - self.last_speech_ms >= self.silence_ms {
                Verdict::Silence
            } else {
                Verdict::Continue
            }
        } else if self.elapsed_ms >= self.no_speech_ms {
            Verdict::NoSpeech
        } else {
            Verdict::Continue
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BLOCK_MS: f32 = 8.0;
    const SPEECH: f32 = 0.08;
    const HUM: f32 = 0.005;

    fn detector() -> SilenceDetector {
        SilenceDetector::new(3000, 10_000)
    }

    /// Feed `ms` of audio at one level; return the first non-continue verdict and when it came.
    fn feed(d: &mut SilenceDetector, level: f32, ms: f32, start_at: f32) -> (Verdict, f32) {
        let mut t = 0.0;
        while t < ms {
            let verdict = d.push(level, BLOCK_MS);
            if verdict != Verdict::Continue {
                return (verdict, start_at + t + BLOCK_MS);
            }
            t += BLOCK_MS;
        }
        (Verdict::Continue, start_at + ms)
    }

    #[test]
    fn ends_after_the_silence_window_once_speech_was_heard() {
        let mut d = detector();
        assert_eq!(feed(&mut d, SPEECH, 2000.0, 0.0).0, Verdict::Continue);
        let (verdict, at) = feed(&mut d, HUM, 5000.0, 2000.0);
        assert_eq!(verdict, Verdict::Silence);
        assert!(at - 2000.0 >= 3000.0 && at - 2000.0 <= 3000.0 + BLOCK_MS);
    }

    #[test]
    fn does_not_end_during_a_short_pause() {
        let mut d = detector();
        feed(&mut d, SPEECH, 1500.0, 0.0);
        assert_eq!(feed(&mut d, HUM, 2500.0, 1500.0).0, Verdict::Continue);
        assert_eq!(feed(&mut d, SPEECH, 1000.0, 4000.0).0, Verdict::Continue);
    }

    #[test]
    fn restarts_the_silence_timer_whenever_speech_resumes() {
        let mut d = detector();
        feed(&mut d, SPEECH, 1000.0, 0.0);
        feed(&mut d, HUM, 2500.0, 1000.0);
        feed(&mut d, SPEECH, 500.0, 3500.0);
        assert_eq!(feed(&mut d, HUM, 2900.0, 4000.0).0, Verdict::Continue);
        assert_eq!(feed(&mut d, HUM, 500.0, 6900.0).0, Verdict::Silence);
    }

    #[test]
    fn gives_up_when_no_speech_is_heard() {
        let mut d = detector();
        let (verdict, at) = feed(&mut d, HUM, 15_000.0, 0.0);
        assert_eq!(verdict, Verdict::NoSpeech);
        assert!(at >= 10_000.0);
    }

    #[test]
    fn ignores_a_short_click_and_still_waits_for_real_speech() {
        let mut d = detector();
        feed(&mut d, HUM, 500.0, 0.0);
        feed(&mut d, 0.3, 40.0, 500.0);
        assert_eq!(feed(&mut d, HUM, 4000.0, 540.0).0, Verdict::Continue);
        assert_eq!(feed(&mut d, HUM, 6000.0, 4540.0).0, Verdict::NoSpeech);
    }

    #[test]
    fn never_ends_while_speech_is_continuous() {
        let mut d = detector();
        assert_eq!(feed(&mut d, SPEECH, 60_000.0, 0.0).0, Verdict::Continue);
    }

    #[test]
    fn treats_brief_dips_between_words_as_part_of_the_same_speech() {
        let mut d = detector();
        let mut verdict = Verdict::Continue;
        let mut i = 0;
        while i < 400 && verdict == Verdict::Continue {
            verdict = d.push(if i % 6 < 4 { SPEECH } else { HUM }, BLOCK_MS);
            i += 1;
        }
        assert_eq!(verdict, Verdict::Continue);
        assert_eq!(feed(&mut d, HUM, 3500.0, 3200.0).0, Verdict::Silence);
    }
}
