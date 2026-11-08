//! The shell connects the pure state machine in `murmur-core` to the real world: it feeds
//! events in, and carries out the effects that come back (microphone, speech engine, typing,
//! commands, the pill window).

use crate::audio::{AudioEvent, Recorder};
use crate::hotkeys::Hotkeys;
use crate::{actions, insert, model, windows};
use murmur_core::controller::{is_valid_audio, Config, Controller, Effect, Event, Mode};
use murmur_core::format::format_transcript;
use murmur_core::memory::{LearningResult, MemoryKind, VoiceMemory};
use murmur_core::settings::Settings;
use murmur_engine::Engine;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{mpsc, Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};

/// How long a spoken confirmation waits before it counts as "no".
const CONFIRM_TIMEOUT: Duration = Duration::from_secs(20);
/// Typing waits this long for a held trigger key to be let go, so the paste is not Alt+Ctrl+V.
const KEY_RELEASE_WAIT: Duration = Duration::from_secs(3);

/// Pause before a command runs so keyboard focus is back in the user's app.
const FOCUS_SETTLE: Duration = Duration::from_millis(150);

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Progress {
    received_bytes: u64,
    total_bytes: u64,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct VoiceTrainingEvent {
    status: &'static str,
    message: Option<String>,
    raw: Option<String>,
    personalized: Option<String>,
}

struct TrainingTranscript {
    raw: String,
    personalized: String,
    applied_entry_ids: Vec<String>,
}

struct PendingLearning {
    id: String,
    inserted: String,
}

/// Lock a mutex even if a thread panicked while holding it; the data is still usable here.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub struct Shell {
    pub app: AppHandle,
    data_dir: PathBuf,
    settings: Mutex<Settings>,
    controller: Mutex<Controller>,
    recorder: Recorder,
    engine: Mutex<Option<Engine>>,
    audio: Mutex<Option<Vec<f32>>>,
    memory: Mutex<crate::memory::MemoryStore>,
    pending_learning: Mutex<Option<PendingLearning>>,
    hotkeys: Hotkeys,
    held_keys: AtomicUsize,
    confirm_token: AtomicU64,
    /// Which recording is current; audio events from older ones are ignored.
    session: AtomicU64,
    /// Effects are carried out by one thread, in the order the state machine produced them.
    effects: mpsc::Sender<Effect>,
    downloading: AtomicBool,
    training_session: AtomicU64,
    started: Instant,
}

fn config_for(settings: &Settings) -> Config {
    Config {
        format: settings.cleanup,
        silence_ms: u32::from(settings.auto_stop_seconds) * 1000,
    }
}

impl Shell {
    pub fn new(app: AppHandle) -> Arc<Shell> {
        // MURMUR_DATA_DIR lets the app run from a different folder (testing, portable use).
        let data_dir = std::env::var_os("MURMUR_DATA_DIR")
            .map(PathBuf::from)
            .or_else(|| app.path().data_dir().ok().map(|dir| dir.join("Murmur")))
            .unwrap_or_else(|| std::env::temp_dir().join("Murmur"));
        let settings = std::fs::read_to_string(data_dir.join("settings.json"))
            .map(|text| Settings::from_json_lenient(&text))
            .unwrap_or_default();
        let memory = crate::memory::MemoryStore::load(&data_dir);

        let (events_tx, events_rx) = mpsc::channel();
        let (effects_tx, effects_rx) = mpsc::channel::<Effect>();
        let shell = Arc::new(Shell {
            session: AtomicU64::new(0),
            effects: effects_tx,
            app,
            controller: Mutex::new(Controller::new(config_for(&settings))),
            settings: Mutex::new(settings),
            data_dir,
            recorder: Recorder::spawn(events_tx),
            engine: Mutex::new(None),
            audio: Mutex::new(None),
            memory: Mutex::new(memory),
            pending_learning: Mutex::new(None),
            hotkeys: Hotkeys::default(),
            held_keys: AtomicUsize::new(0),
            confirm_token: AtomicU64::new(0),
            training_session: AtomicU64::new(0),
            downloading: AtomicBool::new(false),
            started: Instant::now(),
        });
        let forwarder = Arc::clone(&shell);
        let _ = std::thread::Builder::new()
            .name("murmur-audio-events".into())
            .spawn(move || {
                while let Ok(event) = events_rx.recv() {
                    forwarder.on_audio(event);
                }
            });
        let executor = Arc::clone(&shell);
        let _ = std::thread::Builder::new()
            .name("murmur-effects".into())
            .spawn(move || {
                while let Ok(effect) = effects_rx.recv() {
                    executor.run_effect(effect);
                }
            });
        shell
    }

    // ---- settings -------------------------------------------------------------------------

    pub fn settings(&self) -> Settings {
        lock(&self.settings).clone()
    }

    pub fn data_dir(&self) -> &std::path::Path {
        &self.data_dir
    }

    pub fn voice_memory(&self) -> VoiceMemory {
        lock(&self.memory).snapshot()
    }

    pub fn learn_voice_correction(
        &self,
        original: &str,
        corrected: &str,
        context: &str,
    ) -> LearningResult {
        lock(&self.memory).learn_correction(original, corrected, context)
    }

    pub fn add_voice_memory_entry(
        &self,
        canonical: &str,
        aliases: &[String],
        kind: MemoryKind,
        id: Option<&str>,
    ) -> Result<(), String> {
        lock(&self.memory)
            .add_entry(canonical, aliases, kind, id)
            .map(|_| ())
            .ok_or_else(|| "A canonical term is required".into())
    }

    pub fn delete_voice_memory_entry(&self, id: &str) -> bool {
        lock(&self.memory).delete_entry(id)
    }

    pub fn delete_voice_correction(&self, id: &str) -> bool {
        lock(&self.memory).delete_correction(id)
    }

    pub fn clear_voice_memory(&self) -> Result<(), String> {
        lock(&self.memory).clear()
    }

    // ---- voice teaching ------------------------------------------------------------------

    pub fn start_voice_training(self: &Arc<Self>) -> Result<(), String> {
        if !self.model_ready() {
            return Err("Download the speech model before teaching by voice".into());
        }
        if self.training_session.load(Ordering::SeqCst) != 0 {
            return Err("A voice teaching recording is already active".into());
        }

        let session = self.session.fetch_add(1, Ordering::SeqCst) + 1;
        self.training_session.store(session, Ordering::SeqCst);
        self.emit_voice_training(VoiceTrainingEvent {
            status: "recording",
            message: Some("Listening for a teaching example…".into()),
            raw: None,
            personalized: None,
        });
        self.recorder.start(0, session);
        Ok(())
    }

    pub fn stop_voice_training(&self) -> Result<(), String> {
        if self.training_session.load(Ordering::SeqCst) == 0 {
            return Err("No voice teaching recording is active".into());
        }
        self.emit_voice_training(VoiceTrainingEvent {
            status: "processing",
            message: Some("Transcribing your teaching example locally…".into()),
            raw: None,
            personalized: None,
        });
        self.recorder.stop();
        Ok(())
    }

    fn emit_voice_training(&self, event: VoiceTrainingEvent) {
        let _ = self.app.emit("voice-training", event);
    }

    /// Use these settings now (without saving them).
    pub fn apply_settings(&self, settings: Settings) {
        lock(&self.controller).set_config(config_for(&settings));
        *lock(&self.settings) = settings;
    }

    /// Write the current settings to disk.
    pub fn persist_settings(&self) -> Result<(), String> {
        std::fs::create_dir_all(&self.data_dir)
            .map_err(|e| format!("Cannot save settings: {e}"))?;
        let json = serde_json::to_string_pretty(&self.settings()).map_err(|e| e.to_string())?;
        // Write beside the file and rename, so a crash can never leave half a settings file.
        let target = self.data_dir.join("settings.json");
        let temporary = self.data_dir.join("settings.json.tmp");
        std::fs::write(&temporary, json)
            .and_then(|()| std::fs::rename(&temporary, &target))
            .map_err(|e| format!("Cannot save settings: {e}"))
    }

    /// (Re)register the hotkeys. Returns the ones that could not be registered.
    pub fn apply_hotkeys(self: &Arc<Self>) -> Vec<String> {
        self.hotkeys.register(self, &self.settings())
    }

    pub fn key_held(&self, down: bool) {
        if down {
            self.held_keys.fetch_add(1, Ordering::SeqCst);
        } else {
            let _ = self
                .held_keys
                .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |n| n.checked_sub(1));
        }
    }

    // ---- model ----------------------------------------------------------------------------

    fn model_dir(&self) -> PathBuf {
        model::model_dir(&self.data_dir)
    }

    pub fn model_ready(&self) -> bool {
        model::is_ready(&self.model_dir())
    }

    pub fn is_downloading(&self) -> bool {
        self.downloading.load(Ordering::SeqCst)
    }

    /// Download and verify the model, reporting progress to the settings window.
    pub fn download_model(self: &Arc<Self>) -> Result<(), String> {
        if self.downloading.swap(true, Ordering::SeqCst) {
            return Err("A download is already running".into());
        }
        let cancel = AtomicBool::new(false);
        let app = self.app.clone();
        let result = model::download(&self.model_dir(), &cancel, |received, total| {
            let _ = app.emit(
                "model-progress",
                Progress {
                    received_bytes: received,
                    total_bytes: total,
                },
            );
        });
        self.downloading.store(false, Ordering::SeqCst);
        if result.is_ok() {
            *lock(&self.engine) = None;
            self.warm_engine();
        }
        result
    }

    /// Load the speech model in the background so the first recording is fast.
    pub fn warm_engine(self: &Arc<Self>) {
        let shell = Arc::clone(self);
        let _ = std::thread::Builder::new()
            .name("murmur-warm".into())
            .spawn(move || {
                let _ = shell.with_engine(|_| ());
            });
    }

    fn with_engine<T>(&self, work: impl FnOnce(&mut Engine) -> T) -> Result<T, String> {
        let mut guard = lock(&self.engine);
        if guard.is_none() {
            *guard = Some(Engine::load(&self.model_dir())?);
        }
        guard
            .as_mut()
            .map(work)
            .ok_or_else(|| "Speech engine is not available".to_string())
    }

    // ---- events in ------------------------------------------------------------------------

    fn now_ms(&self) -> u64 {
        self.started.elapsed().as_millis() as u64
    }

    pub fn toggle(self: &Arc<Self>, mode: Mode) {
        self.send(Event::Toggle {
            mode,
            model_ready: self.model_ready(),
        });
    }

    pub fn key_down(self: &Arc<Self>, mode: Mode) {
        self.send(Event::KeyDown {
            mode,
            model_ready: self.model_ready(),
            now_ms: self.now_ms(),
        });
    }

    pub fn key_up(self: &Arc<Self>, mode: Mode) {
        self.send(Event::KeyUp {
            mode,
            now_ms: self.now_ms(),
        });
    }

    pub fn cancel(self: &Arc<Self>) {
        self.send(Event::Cancel);
    }

    /// The user answered a confirmation shown in the pill.
    pub fn confirm(self: &Arc<Self>, approved: bool) {
        self.confirm_token.fetch_add(1, Ordering::SeqCst);
        windows::end_confirm(&self.app);
        self.send(Event::Confirmed(approved));
    }

    fn on_audio(self: &Arc<Self>, event: AudioEvent) {
        let current = self.session.load(Ordering::SeqCst);
        match event {
            AudioEvent::Started(id) if self.training_session.load(Ordering::SeqCst) == id => {
                self.emit_voice_training(VoiceTrainingEvent {
                    status: "recording",
                    message: Some(
                        "Microphone is live. Speak the sentence you want to teach…".into(),
                    ),
                    raw: None,
                    personalized: None,
                });
            }
            AudioEvent::Finished(id, samples)
                if self.training_session.swap(0, Ordering::SeqCst) == id =>
            {
                self.emit_voice_training(VoiceTrainingEvent {
                    status: "processing",
                    message: Some("Transcribing your teaching example locally…".into()),
                    raw: None,
                    personalized: None,
                });
                self.spawn("murmur-voice-training", move |shell| {
                    match shell.transcribe_samples(&samples) {
                        Ok(result) => shell.emit_voice_training(VoiceTrainingEvent {
                            status: "completed",
                            message: Some("Transcript ready. Review it, enter what you meant, and teach the correction.".into()),
                            raw: Some(result.raw),
                            personalized: Some(result.personalized),
                        }),
                        Err(error) => shell.emit_voice_training(VoiceTrainingEvent {
                            status: "error",
                            message: Some(error),
                            raw: None,
                            personalized: None,
                        }),
                    }
                });
            }
            AudioEvent::Failed(id, error)
                if self.training_session.swap(0, Ordering::SeqCst) == id =>
            {
                self.emit_voice_training(VoiceTrainingEvent {
                    status: "error",
                    message: Some(error),
                    raw: None,
                    personalized: None,
                });
            }
            AudioEvent::Started(id) if id == current => self.send(Event::CaptureStarted),
            AudioEvent::Finished(id, samples) if id == current => {
                let count = samples.len();
                *lock(&self.audio) = Some(samples);
                self.send(Event::Audio { samples: count });
            }
            AudioEvent::Failed(id, message) if id == current => {
                self.send(Event::CaptureFailed(message))
            }
            _ => {} // left over from a recording that was cancelled or replaced
        }
    }

    /// Feed one event to the state machine and queue what it asks for. Queuing happens while
    /// the controller is locked, so effects can never be reordered between threads.
    pub fn send(self: &Arc<Self>, event: Event) {
        let mut controller = lock(&self.controller);
        for effect in controller.handle(event) {
            let _ = self.effects.send(effect);
        }
    }

    // ---- effects out ----------------------------------------------------------------------

    fn spawn(self: &Arc<Self>, name: &str, work: impl FnOnce(Arc<Shell>) + Send + 'static) {
        let shell = Arc::clone(self);
        let _ = std::thread::Builder::new()
            .name(name.to_string())
            .spawn(move || work(shell));
    }

    fn run_effect(self: &Arc<Self>, effect: Effect) {
        match effect {
            Effect::Publish(state) => windows::publish_pill(&self.app, &state),
            Effect::StartCapture { silence_ms } => {
                let session = self.session.fetch_add(1, Ordering::SeqCst) + 1;
                self.recorder.start(silence_ms, session);
            }
            Effect::StopCapture => self.recorder.stop(),
            Effect::CancelCapture => {
                self.session.fetch_add(1, Ordering::SeqCst);
                self.recorder.cancel();
            }
            Effect::Transcribe => self.spawn("murmur-transcribe", |shell| {
                let result = shell.transcribe();
                shell.send(Event::Transcript(result));
            }),
            Effect::InsertText(text) => self.spawn("murmur-insert", move |shell| {
                shell.wait_for_trigger_key_release();
                let paste_result = insert::paste_text(&text);
                let pasted = paste_result.is_ok();
                shell.send(Event::Inserted(paste_result));
                if pasted {
                    shell.start_learning_observation(text);
                } else {
                    shell.finish_learning_observation(false);
                }
            }),
            Effect::Run(action) => self.spawn("murmur-action", move |shell| {
                shell.wait_for_trigger_key_release();
                // Let focus settle back on the user's app (e.g. after the confirmation card).
                std::thread::sleep(FOCUS_SETTLE);
                let (ok, message) = match actions::run(&action) {
                    Ok(done) => (true, done),
                    Err(failed) => (false, failed),
                };
                shell.send(Event::ActionFinished { ok, message });
            }),
            Effect::Confirm { summary } => {
                let token = self.confirm_token.fetch_add(1, Ordering::SeqCst) + 1;
                windows::show_confirm(&self.app, &summary);
                self.spawn("murmur-confirm-timeout", move |shell| {
                    std::thread::sleep(CONFIRM_TIMEOUT);
                    if shell.confirm_token.load(Ordering::SeqCst) == token {
                        shell.confirm(false);
                    }
                });
            }
        }
    }

    fn transcribe(&self) -> Result<String, String> {
        let samples = lock(&self.audio).take().ok_or("No audio was recorded")?;
        let result = self.transcribe_samples(&samples)?;
        if lock(&self.controller).mode() == Mode::Dictation {
            self.prepare_learning_example(&samples, &result);
        }
        Ok(result.personalized)
    }

    fn prepare_learning_example(&self, samples: &[f32], result: &TrainingTranscript) {
        let id_hint = format!("{}-{}", std::process::id(), self.now_ms());
        let relative_audio = format!("learning-audio/{id_hint}.wav");
        let path = self.data_dir.join(&relative_audio);
        if crate::learning::write_wav(&path, samples).is_err() {
            return;
        }
        let id = lock(&self.memory).add_learning_example(
            &result.raw,
            &result.personalized,
            &relative_audio,
            &result.applied_entry_ids,
        );
        *lock(&self.pending_learning) = Some(PendingLearning {
            id,
            inserted: result.personalized.clone(),
        });
    }

    fn start_learning_observation(self: &Arc<Self>, inserted: String) {
        let pending = lock(&self.pending_learning).take();
        let Some(pending) = pending else {
            return;
        };
        if pending.inserted != inserted {
            let _ = lock(&self.memory).mark_learning_observation_unavailable(&pending.id);
            let _ = self.app.emit("learning-update", ());
            return;
        }
        let shell = Arc::clone(self);
        let _ = std::thread::Builder::new()
            .name("murmur-learning-observer".into())
            .spawn(move || {
                // The paste helper waits for the target application to consume the clipboard, but
                // UI Automation can report the editor a little later. Wait for the inserted text to
                // become observable before starting the edit window; otherwise the first snapshot
                // can be taken too early and every correction is lost.
                let baseline_deadline = Instant::now() + Duration::from_secs(3);
                let baseline = loop {
                    if let Some(snapshot) = crate::target_observer::read_focused_text() {
                        if crate::target_observer::contains_inserted_text(
                            &snapshot.text,
                            &pending.inserted,
                        ) {
                            break Some(snapshot);
                        }
                    }
                    if Instant::now() >= baseline_deadline {
                        break None;
                    }
                    std::thread::sleep(Duration::from_millis(100));
                };
                let Some(baseline) = baseline else {
                    // An editor that does not expose UI Automation is not evidence that the
                    // correction failed. Keep the example auditable without poisoning its failure
                    // count.
                    let _ = lock(&shell.memory).mark_learning_observation_unavailable(&pending.id);
                    let _ = shell.app.emit("learning-update", ());
                    return;
                };
                let deadline = Instant::now() + Duration::from_secs(20);
                let mut unrelated_edit = false;
                let mut observed_same_editor = false;
                while Instant::now() < deadline {
                    std::thread::sleep(Duration::from_millis(350));
                    let Some(current) = crate::target_observer::read_focused_text() else {
                        continue;
                    };
                    if current.identity != baseline.identity {
                        continue;
                    }
                    observed_same_editor = true;
                    if current.text != baseline.text {
                        if let Some(corrected) = crate::target_observer::extract_edited_phrase(
                            &baseline.text,
                            &current.text,
                            &pending.inserted,
                        ) {
                            let result = lock(&shell.memory).complete_learning_example(
                                &pending.id,
                                &corrected,
                                "post-dictation edit",
                            );
                            let _ = shell.app.emit("learning-update", result);
                            return;
                        }
                        unrelated_edit = true;
                    }
                }
                let accepted = observed_same_editor && !unrelated_edit;
                let _ = lock(&shell.memory).mark_learning_outcome(&pending.id, accepted);
                let _ = shell.app.emit("learning-update", ());
            });
    }

    fn finish_learning_observation(&self, success: bool) {
        let pending = lock(&self.pending_learning).take();
        if let Some(pending) = pending {
            if success {
                let _ = lock(&self.memory).mark_learning_outcome(&pending.id, true);
            } else {
                let _ = lock(&self.memory).mark_learning_observation_unavailable(&pending.id);
            }
            let _ = self.app.emit("learning-update", ());
        }
    }

    fn transcribe_samples(&self, samples: &[f32]) -> Result<TrainingTranscript, String> {
        if !is_valid_audio(&samples) {
            return Err("The recording was not valid audio".into());
        }
        let raw = self.with_engine(|engine| engine.transcribe(samples))?;
        let settings = self.settings();
        let cleaned = format_transcript(&raw, settings.cleanup);
        let context = {
            let memory = lock(&self.memory).snapshot();
            memory
                .recent_transcripts
                .iter()
                .take(3)
                .map(|record| record.personalized.as_str())
                .collect::<Vec<_>>()
                .join(" ")
        };
        let mode = lock(&self.controller).mode();
        let personalized = {
            let mut memory = lock(&self.memory);
            let result = memory.personalize(&cleaned, &context);
            memory.record_transcript(
                &raw,
                &result.text,
                match mode {
                    Mode::Dictation => "dictation",
                    Mode::Agent => "agent",
                },
            );
            TrainingTranscript {
                raw,
                personalized: result.text,
                applied_entry_ids: result.applied_entry_ids,
            }
        };
        Ok(personalized)
    }

    /// If the trigger key is still held (push-to-talk with auto-finish), wait for it, so the
    /// paste or shortcut is not sent with Alt or Ctrl still down.
    fn wait_for_trigger_key_release(&self) {
        let deadline = Instant::now() + KEY_RELEASE_WAIT;
        while self.held_keys.load(Ordering::SeqCst) > 0 && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(20));
        }
    }
}
