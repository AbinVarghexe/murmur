//! The recording state machine: hotkey → recording → transcription → typed text or command.
//!
//! It is pure: `handle(event)` returns the effects the app should carry out (start the
//! microphone, type text, run a command…). No threads, timers or operating-system calls, so
//! every scenario is a plain unit test and behaves the same on Windows and macOS.

use crate::format::{format_transcript, FormatOptions};
use crate::planner::{plan_command, Action};
use serde::Serialize;

pub const SAMPLE_RATE: u32 = 16_000;
/// Recordings shorter than this (0.3 s) are discarded.
pub const MIN_SAMPLES: usize = 4_800;
/// Safety cap for one recording (30 minutes). Keeps an unattended mic from filling memory.
pub const MAX_RECORDING_SECONDS: usize = 30 * 60;
pub const MAX_SAMPLES: usize = SAMPLE_RATE as usize * MAX_RECORDING_SECONDS;
/// Holding a single-key hotkey at least this long means push-to-talk; a shorter tap latches.
pub const HOLD_MS: u64 = 350;
const PREVIEW_CHARS: usize = 60;

/// Accept only plausible audio: bounded length, finite values.
pub fn is_valid_audio(samples: &[f32]) -> bool {
    samples.len() <= MAX_SAMPLES && samples.iter().all(|s| s.is_finite())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Dictation,
    Agent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Phase {
    Idle,
    /// The microphone is opening. Nothing is being recorded yet.
    Preparing,
    Recording,
    Processing,
    /// A finished action or typed text.
    Result,
    /// Something did not work, or nothing happened.
    Notice,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PillState {
    pub phase: Phase,
    pub mode: Mode,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// A chord hotkey was pressed.
    Toggle {
        mode: Mode,
        model_ready: bool,
    },
    /// A single-key hotkey went down or up (push-to-talk or tap-to-toggle).
    KeyDown {
        mode: Mode,
        model_ready: bool,
        now_ms: u64,
    },
    KeyUp {
        mode: Mode,
        now_ms: u64,
    },
    Cancel,
    /// The microphone is open and audio is flowing.
    CaptureStarted,
    CaptureFailed(String),
    /// The recorder finished and delivered this many samples.
    Audio {
        samples: usize,
    },
    Transcript(Result<String, String>),
    Inserted(Result<(), String>),
    Confirmed(bool),
    /// A command ran (`ok`) or failed, with a message for the pill.
    ActionFinished {
        ok: bool,
        message: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    Publish(PillState),
    StartCapture {
        silence_ms: u32,
    },
    /// Finish recording and deliver the audio.
    StopCapture,
    /// Stop recording and throw the audio away.
    CancelCapture,
    /// Run the speech model on the audio just delivered.
    Transcribe,
    InsertText(String),
    Run(Action),
    /// Ask the user before running a disruptive command.
    Confirm {
        summary: String,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub format: FormatOptions,
    /// Quiet time after speech that ends a recording by itself. 0 means never.
    pub silence_ms: u32,
}

#[derive(Debug, Clone)]
enum Busy {
    Idle,
    /// Waiting for the microphone. `stop_when_ready` remembers a stop that arrived early.
    Starting {
        stop_when_ready: bool,
    },
    Recording,
    Processing,
    Inserting {
        message: String,
    },
    Confirming(Action),
    Running,
}

pub struct Controller {
    busy: Busy,
    mode: Mode,
    press_started_at: Option<u64>,
    config: Config,
}

impl Controller {
    pub fn new(config: Config) -> Self {
        Self {
            busy: Busy::Idle,
            mode: Mode::Dictation,
            press_started_at: None,
            config,
        }
    }

    pub fn set_config(&mut self, config: Config) {
        self.config = config;
    }

    pub fn is_idle(&self) -> bool {
        matches!(self.busy, Busy::Idle)
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    fn state(&self, phase: Phase, message: impl Into<String>) -> Effect {
        Effect::Publish(PillState {
            phase,
            mode: self.mode,
            message: message.into(),
        })
    }

    /// Something worked: show the outcome.
    fn finish(&mut self, message: impl Into<String>) -> Vec<Effect> {
        self.busy = Busy::Idle;
        vec![self.state(Phase::Result, message)]
    }

    /// Something did not work (or did not happen): show why, styled differently from a success.
    fn notice(&mut self, message: impl Into<String>) -> Vec<Effect> {
        self.busy = Busy::Idle;
        vec![self.state(Phase::Notice, message)]
    }

    fn is_capturing(&self) -> bool {
        matches!(self.busy, Busy::Starting { .. } | Busy::Recording)
    }

    /// Ask the recorder to finish: now if it is recording, or as soon as it has started.
    fn request_stop(&mut self) -> Vec<Effect> {
        match &mut self.busy {
            Busy::Recording => vec![Effect::StopCapture],
            Busy::Starting { stop_when_ready } => {
                *stop_when_ready = true;
                Vec::new()
            }
            _ => Vec::new(),
        }
    }

    fn toggle(&mut self, mode: Mode, model_ready: bool) -> Vec<Effect> {
        match self.busy {
            Busy::Processing | Busy::Inserting { .. } | Busy::Confirming(_) | Busy::Running => {
                Vec::new()
            }
            Busy::Recording | Busy::Starting { .. } => {
                if mode == self.mode {
                    self.request_stop()
                } else {
                    Vec::new()
                }
            }
            Busy::Idle => {
                self.mode = mode;
                if !model_ready {
                    return self
                        .notice("Speech model not installed. Open Settings to download it.");
                }
                self.busy = Busy::Starting {
                    stop_when_ready: false,
                };
                vec![
                    self.state(Phase::Preparing, "Getting ready…"),
                    Effect::StartCapture {
                        silence_ms: self.config.silence_ms,
                    },
                ]
            }
        }
    }

    pub fn handle(&mut self, event: Event) -> Vec<Effect> {
        match event {
            Event::Toggle { mode, model_ready } => self.toggle(mode, model_ready),
            Event::KeyDown {
                mode,
                model_ready,
                now_ms,
            } => {
                if self.is_capturing() && self.mode == mode {
                    self.press_started_at = None;
                    return self.request_stop();
                }
                let was_idle = matches!(self.busy, Busy::Idle);
                let effects = self.toggle(mode, model_ready);
                if was_idle && matches!(self.busy, Busy::Starting { .. }) {
                    self.press_started_at = Some(now_ms);
                }
                effects
            }
            Event::KeyUp { mode, now_ms } => {
                if self.mode != mode {
                    return Vec::new(); // another key's release must not end this recording
                }
                let started = self.press_started_at.take();
                match started {
                    Some(at)
                        if self.mode == mode
                            && self.is_capturing()
                            && now_ms.saturating_sub(at) >= HOLD_MS =>
                    {
                        self.request_stop()
                    }
                    _ => Vec::new(),
                }
            }
            Event::Cancel => {
                if !self.is_capturing() {
                    return Vec::new();
                }
                self.busy = Busy::Idle;
                self.press_started_at = None;
                vec![Effect::CancelCapture, self.state(Phase::Idle, "")]
            }
            Event::CaptureStarted => self.on_capture_started(),
            Event::CaptureFailed(message) => {
                if self.is_capturing() {
                    self.notice(format!("Microphone error: {message}"))
                } else {
                    Vec::new()
                }
            }
            Event::Audio { samples } => self.on_audio(samples),
            Event::Transcript(result) => self.on_transcript(result),
            Event::Inserted(result) => self.on_inserted(result),
            Event::Confirmed(approved) => self.on_confirmed(approved),
            Event::ActionFinished { ok, message } => {
                if !matches!(self.busy, Busy::Running) {
                    Vec::new()
                } else if ok {
                    self.finish(message)
                } else {
                    self.notice(message)
                }
            }
        }
    }

    fn on_capture_started(&mut self) -> Vec<Effect> {
        let Busy::Starting { stop_when_ready } = self.busy else {
            return Vec::new();
        };
        self.busy = Busy::Recording;
        let label = match self.mode {
            Mode::Agent => "Listening for a command…",
            Mode::Dictation => "Listening…",
        };
        let mut effects = vec![self.state(Phase::Recording, label)];
        if stop_when_ready {
            effects.push(Effect::StopCapture);
        }
        effects
    }

    fn on_audio(&mut self, samples: usize) -> Vec<Effect> {
        if !self.is_capturing() {
            return Vec::new();
        }
        if samples < MIN_SAMPLES {
            return self.notice("Too short. Try again.");
        }
        self.busy = Busy::Processing;
        vec![
            self.state(Phase::Processing, "Transcribing…"),
            Effect::Transcribe,
        ]
    }

    fn on_transcript(&mut self, result: Result<String, String>) -> Vec<Effect> {
        if !matches!(self.busy, Busy::Processing) {
            return Vec::new();
        }
        let raw = match result {
            Ok(text) => text.trim().to_string(),
            Err(message) => return self.notice(message),
        };
        if raw.is_empty() {
            return self.notice("Didn't catch that.");
        }
        match self.mode {
            Mode::Dictation => self.dictate(&raw),
            Mode::Agent => self.run_command(&raw),
        }
    }

    fn dictate(&mut self, raw: &str) -> Vec<Effect> {
        let text = format_transcript(raw, self.config.format);
        if text.is_empty() {
            return self.notice("Didn't catch that.");
        }
        let message = if text.chars().count() > PREVIEW_CHARS {
            format!(
                "{}…",
                text.chars().take(PREVIEW_CHARS - 3).collect::<String>()
            )
        } else {
            text.clone()
        };
        self.busy = Busy::Inserting { message };
        vec![Effect::InsertText(text)]
    }

    fn run_command(&mut self, raw: &str) -> Vec<Effect> {
        let Some(plan) = plan_command(raw) else {
            return self.notice(format!("Not a command I know: \"{raw}\""));
        };
        if plan.confirm {
            self.busy = Busy::Confirming(plan.action);
            return vec![
                self.state(Phase::Processing, "Waiting for your OK…"),
                Effect::Confirm {
                    summary: plan.summary,
                },
            ];
        }
        self.busy = Busy::Running;
        vec![Effect::Run(plan.action)]
    }

    fn on_inserted(&mut self, result: Result<(), String>) -> Vec<Effect> {
        let Busy::Inserting { message } = self.busy.clone() else {
            return Vec::new();
        };
        match result {
            Ok(()) => self.finish(message),
            Err(error) => self.notice(error),
        }
    }

    fn on_confirmed(&mut self, approved: bool) -> Vec<Effect> {
        let Busy::Confirming(action) = self.busy.clone() else {
            return Vec::new();
        };
        if approved {
            self.busy = Busy::Running;
            vec![Effect::Run(action)]
        } else {
            self.notice("Cancelled.")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planner::WindowOp;

    const ONE_SECOND: usize = SAMPLE_RATE as usize;

    fn controller() -> Controller {
        Controller::new(Config {
            format: FormatOptions::default(),
            silence_ms: 3000,
        })
    }

    fn toggle(mode: Mode) -> Event {
        Event::Toggle {
            mode,
            model_ready: true,
        }
    }

    fn down(mode: Mode, now_ms: u64) -> Event {
        Event::KeyDown {
            mode,
            model_ready: true,
            now_ms,
        }
    }

    fn last_message(effects: &[Effect]) -> Option<String> {
        effects.iter().rev().find_map(|e| match e {
            Effect::Publish(state) => Some(state.message.clone()),
            _ => None,
        })
    }

    fn last_phase(effects: &[Effect]) -> Option<Phase> {
        effects.iter().rev().find_map(|e| match e {
            Effect::Publish(state) => Some(state.phase),
            _ => None,
        })
    }

    /// Start recording and get the microphone live.
    fn recording(c: &mut Controller, mode: Mode) {
        c.handle(toggle(mode));
        c.handle(Event::CaptureStarted);
    }

    fn deliver(c: &mut Controller, text: &str) -> Vec<Effect> {
        c.handle(Event::Audio {
            samples: ONE_SECOND,
        });
        c.handle(Event::Transcript(Ok(text.into())))
    }

    #[test]
    fn dictation_flow_cleans_the_text_and_reports_it() {
        let mut c = controller();
        let start = c.handle(toggle(Mode::Dictation));
        assert!(start.contains(&Effect::StartCapture { silence_ms: 3000 }));
        assert_eq!(last_phase(&start), Some(Phase::Preparing));

        let live = c.handle(Event::CaptureStarted);
        assert_eq!(last_phase(&live), Some(Phase::Recording));

        let audio = c.handle(Event::Audio {
            samples: ONE_SECOND,
        });
        assert!(audio.contains(&Effect::Transcribe));

        let typed = c.handle(Event::Transcript(Ok("um hello world".into())));
        assert_eq!(typed, vec![Effect::InsertText("Hello world".into())]);

        let done = c.handle(Event::Inserted(Ok(())));
        assert_eq!(last_message(&done).as_deref(), Some("Hello world"));
        assert!(c.is_idle());
    }

    #[test]
    fn says_getting_ready_until_the_microphone_is_live() {
        let mut c = controller();
        let effects = c.handle(toggle(Mode::Dictation));
        assert_eq!(last_message(&effects).as_deref(), Some("Getting ready…"));
        let live = c.handle(Event::CaptureStarted);
        assert_eq!(last_message(&live).as_deref(), Some("Listening…"));
    }

    #[test]
    fn stops_capture_when_the_same_hotkey_is_pressed_again() {
        let mut c = controller();
        recording(&mut c, Mode::Dictation);
        assert_eq!(c.handle(toggle(Mode::Dictation)), vec![Effect::StopCapture]);
    }

    #[test]
    fn ignores_the_other_hotkey_while_recording() {
        let mut c = controller();
        recording(&mut c, Mode::Dictation);
        assert!(c.handle(toggle(Mode::Agent)).is_empty());
    }

    #[test]
    fn a_stop_that_arrives_while_the_mic_is_opening_is_remembered() {
        let mut c = controller();
        c.handle(toggle(Mode::Dictation));
        assert!(c.handle(toggle(Mode::Dictation)).is_empty());
        let live = c.handle(Event::CaptureStarted);
        assert!(live.contains(&Effect::StopCapture));
    }

    #[test]
    fn refuses_to_record_when_the_model_is_missing() {
        let mut c = controller();
        let effects = c.handle(Event::Toggle {
            mode: Mode::Dictation,
            model_ready: false,
        });
        assert!(!effects
            .iter()
            .any(|e| matches!(e, Effect::StartCapture { .. })));
        assert!(last_message(&effects).unwrap().contains("not installed"));
        assert!(c.is_idle());
    }

    #[test]
    fn runs_a_voice_command_in_agent_mode() {
        let mut c = controller();
        recording(&mut c, Mode::Agent);
        let effects = deliver(&mut c, "open notepad");
        assert_eq!(
            effects,
            vec![Effect::Run(Action::OpenApp {
                name: "notepad".into()
            })]
        );
        let done = c.handle(Event::ActionFinished {
            ok: true,
            message: "Opened notepad".into(),
        });
        assert_eq!(last_message(&done).as_deref(), Some("Opened notepad"));
        assert_eq!(last_phase(&done), Some(Phase::Result));
        assert!(c.is_idle());
    }

    #[test]
    fn a_failed_command_is_a_notice_not_a_success() {
        let mut c = controller();
        recording(&mut c, Mode::Agent);
        deliver(&mut c, "open nosuchapp");
        let done = c.handle(Event::ActionFinished {
            ok: false,
            message: "No app named \"nosuchapp\" found".into(),
        });
        assert_eq!(last_phase(&done), Some(Phase::Notice));
        assert!(c.is_idle());
    }

    #[test]
    fn problems_are_notices_and_outcomes_are_results() {
        let mut c = controller();
        let missing = c.handle(Event::Toggle {
            mode: Mode::Dictation,
            model_ready: false,
        });
        assert_eq!(last_phase(&missing), Some(Phase::Notice));

        recording(&mut c, Mode::Dictation);
        let short = c.handle(Event::Audio { samples: 100 });
        assert_eq!(last_phase(&short), Some(Phase::Notice));

        recording(&mut c, Mode::Dictation);
        deliver(&mut c, "hello there");
        let typed = c.handle(Event::Inserted(Ok(())));
        assert_eq!(last_phase(&typed), Some(Phase::Result));
    }

    #[test]
    fn asks_before_disruptive_commands_and_respects_a_refusal() {
        let mut c = controller();
        recording(&mut c, Mode::Agent);
        let effects = deliver(&mut c, "close window");
        assert!(effects.iter().any(|e| matches!(e, Effect::Confirm { .. })));
        assert!(!effects.iter().any(|e| matches!(e, Effect::Run(_))));
        let refused = c.handle(Event::Confirmed(false));
        assert_eq!(last_message(&refused).as_deref(), Some("Cancelled."));
    }

    #[test]
    fn runs_a_disruptive_command_once_approved() {
        let mut c = controller();
        recording(&mut c, Mode::Agent);
        deliver(&mut c, "close window");
        let approved = c.handle(Event::Confirmed(true));
        assert_eq!(
            approved,
            vec![Effect::Run(Action::Window {
                op: WindowOp::Close
            })]
        );
    }

    #[test]
    fn reports_unknown_commands_without_acting() {
        let mut c = controller();
        recording(&mut c, Mode::Agent);
        let effects = deliver(&mut c, "the quick brown fox");
        assert!(!effects.iter().any(|e| matches!(e, Effect::Run(_))));
        assert!(last_message(&effects).unwrap().starts_with("Not a command"));
    }

    #[test]
    fn discards_very_short_recordings() {
        let mut c = controller();
        recording(&mut c, Mode::Dictation);
        let effects = c.handle(Event::Audio { samples: 100 });
        assert!(!effects.contains(&Effect::Transcribe));
        assert_eq!(
            last_message(&effects).as_deref(),
            Some("Too short. Try again.")
        );
    }

    #[test]
    fn recovers_to_idle_after_a_transcription_error() {
        let mut c = controller();
        recording(&mut c, Mode::Dictation);
        c.handle(Event::Audio {
            samples: ONE_SECOND,
        });
        let effects = c.handle(Event::Transcript(Err("engine down".into())));
        assert_eq!(last_message(&effects).as_deref(), Some("engine down"));
        assert!(!c.handle(toggle(Mode::Dictation)).is_empty());
    }

    #[test]
    fn says_so_when_nothing_was_heard() {
        let mut c = controller();
        recording(&mut c, Mode::Dictation);
        let effects = deliver(&mut c, "   ");
        assert_eq!(
            last_message(&effects).as_deref(),
            Some("Didn't catch that.")
        );
    }

    #[test]
    fn reports_a_failed_paste() {
        let mut c = controller();
        recording(&mut c, Mode::Dictation);
        deliver(&mut c, "hello there");
        let effects = c.handle(Event::Inserted(Err(
            "Could not paste into the active app".into()
        )));
        assert_eq!(
            last_message(&effects).as_deref(),
            Some("Could not paste into the active app")
        );
    }

    #[test]
    fn long_text_is_shortened_in_the_pill_but_typed_in_full() {
        let mut c = controller();
        recording(&mut c, Mode::Dictation);
        let long = "word ".repeat(40);
        let effects = deliver(&mut c, &long);
        match &effects[0] {
            Effect::InsertText(text) => assert!(text.len() > 100),
            other => panic!("unexpected {other:?}"),
        }
        let done = c.handle(Event::Inserted(Ok(())));
        let message = last_message(&done).unwrap();
        assert!(message.chars().count() <= PREVIEW_CHARS && message.ends_with('…'));
    }

    #[test]
    fn two_quick_presses_start_only_one_recording() {
        let mut c = controller();
        let first = c.handle(toggle(Mode::Dictation));
        let second = c.handle(toggle(Mode::Dictation));
        assert_eq!(
            first
                .iter()
                .filter(|e| matches!(e, Effect::StartCapture { .. }))
                .count(),
            1
        );
        assert!(!second
            .iter()
            .any(|e| matches!(e, Effect::StartCapture { .. })));
    }

    #[test]
    fn ignores_a_stale_capture_error_when_not_recording() {
        let mut c = controller();
        assert!(c.handle(Event::CaptureFailed("late".into())).is_empty());
    }

    #[test]
    fn reports_a_microphone_error_while_starting() {
        let mut c = controller();
        c.handle(toggle(Mode::Dictation));
        let effects = c.handle(Event::CaptureFailed("no device".into()));
        assert_eq!(
            last_message(&effects).as_deref(),
            Some("Microphone error: no device")
        );
        assert!(c.is_idle());
    }

    #[test]
    fn cancel_stops_capture_and_discards_the_audio() {
        let mut c = controller();
        recording(&mut c, Mode::Dictation);
        let effects = c.handle(Event::Cancel);
        assert!(effects.contains(&Effect::CancelCapture));
        assert_eq!(last_phase(&effects), Some(Phase::Idle));
        assert!(c
            .handle(Event::Audio {
                samples: ONE_SECOND
            })
            .is_empty());
    }

    #[test]
    fn push_to_talk_holding_a_single_key_records_and_releasing_stops() {
        let mut c = controller();
        let start = c.handle(down(Mode::Dictation, 1_000));
        assert!(start.contains(&Effect::StartCapture { silence_ms: 3000 }));
        c.handle(Event::CaptureStarted);
        let stop = c.handle(Event::KeyUp {
            mode: Mode::Dictation,
            now_ms: 1_000 + HOLD_MS + 200,
        });
        assert_eq!(stop, vec![Effect::StopCapture]);
    }

    #[test]
    fn a_quick_tap_latches_recording_and_the_next_tap_stops_it() {
        let mut c = controller();
        c.handle(down(Mode::Dictation, 1_000));
        c.handle(Event::CaptureStarted);
        assert!(c
            .handle(Event::KeyUp {
                mode: Mode::Dictation,
                now_ms: 1_080
            })
            .is_empty());
        assert_eq!(
            c.handle(down(Mode::Dictation, 5_000)),
            vec![Effect::StopCapture]
        );
        assert!(c
            .handle(Event::KeyUp {
                mode: Mode::Dictation,
                now_ms: 5_050
            })
            .is_empty());
    }

    #[test]
    fn releasing_the_key_of_a_different_mode_does_not_stop_recording() {
        let mut c = controller();
        c.handle(down(Mode::Dictation, 1_000));
        c.handle(Event::CaptureStarted);
        assert!(c
            .handle(Event::KeyUp {
                mode: Mode::Agent,
                now_ms: 3_000
            })
            .is_empty());
    }

    #[test]
    fn a_stray_release_with_no_press_does_nothing() {
        let mut c = controller();
        assert!(c
            .handle(Event::KeyUp {
                mode: Mode::Dictation,
                now_ms: 100
            })
            .is_empty());
    }

    #[test]
    fn holding_while_the_mic_is_still_opening_stops_as_soon_as_it_is_live() {
        let mut c = controller();
        c.handle(down(Mode::Dictation, 1_000));
        assert!(c
            .handle(Event::KeyUp {
                mode: Mode::Dictation,
                now_ms: 1_000 + HOLD_MS + 100
            })
            .is_empty());
        assert!(c
            .handle(Event::CaptureStarted)
            .contains(&Effect::StopCapture));
    }

    #[test]
    fn hold_speak_keep_holding_then_auto_finish_transcribes_and_the_later_release_does_nothing() {
        let mut c = controller();
        c.handle(down(Mode::Dictation, 1_000));
        c.handle(Event::CaptureStarted);
        // The recorder noticed the pause and ended the recording by itself.
        let typed = deliver(&mut c, "hello world");
        assert_eq!(typed, vec![Effect::InsertText("Hello world".into())]);
        c.handle(Event::Inserted(Ok(())));
        assert!(c
            .handle(Event::KeyUp {
                mode: Mode::Dictation,
                now_ms: 9_000
            })
            .is_empty());
    }

    #[test]
    fn tap_speak_pause_auto_finish_needs_no_second_tap() {
        let mut c = controller();
        c.handle(down(Mode::Dictation, 1_000));
        c.handle(Event::CaptureStarted);
        c.handle(Event::KeyUp {
            mode: Mode::Dictation,
            now_ms: 1_080,
        });
        let typed = deliver(&mut c, "hello world");
        assert_eq!(typed, vec![Effect::InsertText("Hello world".into())]);
    }

    #[test]
    fn passes_the_configured_pause_length_to_the_recorder() {
        let mut c = Controller::new(Config {
            format: FormatOptions::default(),
            silence_ms: 4000,
        });
        assert!(c
            .handle(toggle(Mode::Agent))
            .contains(&Effect::StartCapture { silence_ms: 4000 }));
        let mut off = Controller::new(Config {
            format: FormatOptions::default(),
            silence_ms: 0,
        });
        assert!(off
            .handle(toggle(Mode::Agent))
            .contains(&Effect::StartCapture { silence_ms: 0 }));
    }

    #[test]
    fn a_new_hotkey_press_is_ignored_while_processing() {
        let mut c = controller();
        recording(&mut c, Mode::Dictation);
        c.handle(Event::Audio {
            samples: ONE_SECOND,
        });
        assert!(c.handle(toggle(Mode::Dictation)).is_empty());
    }

    #[test]
    fn rejects_audio_that_is_too_long_or_not_finite() {
        assert!(is_valid_audio(&[0.1, -0.2]));
        assert!(!is_valid_audio(&[0.1, f32::NAN]));
        assert!(!is_valid_audio(&[f32::INFINITY]));
        assert!(!is_valid_audio(&vec![0.0; MAX_SAMPLES + 1]));
    }
}
