//! Microphone capture on its own thread.
//!
//! Opens the default input device, converts whatever it delivers to 16 kHz mono, watches the
//! level for the pause detector, and hands back the finished recording. The device is only
//! open while recording; nothing is kept or sent anywhere.

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream, StreamConfig};
use murmur_core::audio_utils::{lift_if_quiet, rms, to_mono};
use murmur_core::controller::{MAX_SAMPLES, SAMPLE_RATE};
use murmur_core::resample::Resampler;
use murmur_core::vad::{SilenceDetector, Verdict};
use std::sync::mpsc::{self, Receiver, Sender};

/// If nothing is said for this long after starting, the recording ends.
const NO_SPEECH_MS: u32 = 10_000;
/// Recordings quieter than this peak get a volume lift; louder ones are left untouched.
const QUIET_PEAK: f32 = 0.25;
const LIFT_TARGET_PEAK: f32 = 0.5;
const LIFT_MAX_GAIN: f32 = 10.0;

/// Every event names the recording (session) it belongs to, so a late event from a cancelled
/// recording can never be mistaken for the current one.
#[derive(Debug)]
pub enum AudioEvent {
    /// Audio is flowing, so recording has really begun.
    Started(u64),
    /// The finished 16 kHz mono recording.
    Finished(u64, Vec<f32>),
    Failed(u64, String),
}

enum Msg {
    Start { silence_ms: u32, session: u64 },
    Stop,
    Cancel,
    Chunk(Vec<f32>),
    StreamError(String),
}

#[derive(Clone)]
pub struct Recorder {
    tx: Sender<Msg>,
}

impl Recorder {
    pub fn spawn(events: Sender<AudioEvent>) -> Recorder {
        let (tx, rx) = mpsc::channel();
        let worker_tx = tx.clone();
        std::thread::Builder::new()
            .name("murmur-audio".into())
            .spawn(move || worker(rx, worker_tx, events))
            .expect("the audio thread can be started");
        Recorder { tx }
    }

    pub fn start(&self, silence_ms: u32, session: u64) {
        let _ = self.tx.send(Msg::Start {
            silence_ms,
            session,
        });
    }

    /// Finish recording and deliver the audio.
    pub fn stop(&self) {
        let _ = self.tx.send(Msg::Stop);
    }

    /// Stop and throw the audio away.
    pub fn cancel(&self) {
        let _ = self.tx.send(Msg::Cancel);
    }
}

struct Active {
    /// Dropping the stream closes the microphone.
    _stream: Stream,
    resampler: Resampler,
    channels: usize,
    samples: Vec<f32>,
    detector: Option<SilenceDetector>,
    started: bool,
    session: u64,
}

fn worker(rx: Receiver<Msg>, tx: Sender<Msg>, events: Sender<AudioEvent>) {
    let mut active: Option<Active> = None;
    while let Ok(message) = rx.recv() {
        match message {
            Msg::Start {
                silence_ms,
                session,
            } => {
                // A new recording replaces any leftover one (dropping it closes the mic).
                active = None;
                match open(&tx, silence_ms, session) {
                    Ok(recording) => active = Some(recording),
                    Err(error) => {
                        let _ = events.send(AudioEvent::Failed(session, error));
                    }
                }
            }
            Msg::Stop => {
                if let Some(recording) = active.take() {
                    finish(recording, &events);
                }
            }
            Msg::Cancel => active = None,
            Msg::Chunk(data) => {
                let ended = match active.as_mut() {
                    Some(recording) => feed(recording, &data, &events),
                    None => false,
                };
                if ended {
                    if let Some(recording) = active.take() {
                        finish(recording, &events);
                    }
                }
            }
            Msg::StreamError(error) => {
                if let Some(recording) = active.take() {
                    let _ = events.send(AudioEvent::Failed(recording.session, error));
                }
            }
        }
    }
}

/// Process one block of microphone audio. Returns true when the recording should end.
fn feed(recording: &mut Active, data: &[f32], events: &Sender<AudioEvent>) -> bool {
    let mono = to_mono(data, recording.channels);
    let mut converted = Vec::with_capacity(mono.len() / 2 + 8);
    recording.resampler.push(&mono, &mut converted);
    if converted.is_empty() {
        return false;
    }
    if !recording.started {
        recording.started = true;
        let _ = events.send(AudioEvent::Started(recording.session));
    }
    let level = rms(&converted);
    let block_ms = converted.len() as f32 * 1000.0 / SAMPLE_RATE as f32;
    recording.samples.extend_from_slice(&converted);

    if recording.samples.len() >= MAX_SAMPLES {
        return true;
    }
    match recording.detector.as_mut() {
        Some(detector) => detector.push(level, block_ms) != Verdict::Continue,
        None => false,
    }
}

fn finish(recording: Active, events: &Sender<AudioEvent>) {
    let Active {
        mut samples,
        session,
        ..
    } = recording; // the stream is dropped here, closing the device
    samples.truncate(MAX_SAMPLES); // the last block may overshoot the cap
    let samples = lift_if_quiet(&samples, QUIET_PEAK, LIFT_TARGET_PEAK, LIFT_MAX_GAIN);
    let _ = events.send(AudioEvent::Finished(session, samples));
}

fn open(tx: &Sender<Msg>, silence_ms: u32, session: u64) -> Result<Active, String> {
    let host = cpal::default_host();
    let device = host.default_input_device().ok_or("No microphone found")?;
    let supported = device
        .default_input_config()
        .map_err(|e| format!("Cannot read the microphone settings: {e}"))?;
    let channels = supported.channels() as usize;
    let rate = supported.sample_rate().0;
    let format = supported.sample_format();
    let config: StreamConfig = supported.into();

    let error_tx = tx.clone();
    let on_error = move |error: cpal::StreamError| {
        let _ = error_tx.send(Msg::StreamError(error.to_string()));
    };
    let data_tx = tx.clone();

    let stream = match format {
        SampleFormat::F32 => device.build_input_stream(
            &config,
            move |data: &[f32], _| {
                let _ = data_tx.send(Msg::Chunk(data.to_vec()));
            },
            on_error,
            None,
        ),
        SampleFormat::I16 => device.build_input_stream(
            &config,
            move |data: &[i16], _| {
                let _ = data_tx.send(Msg::Chunk(
                    data.iter().map(|s| f32::from(*s) / 32768.0).collect(),
                ));
            },
            on_error,
            None,
        ),
        SampleFormat::U16 => device.build_input_stream(
            &config,
            move |data: &[u16], _| {
                let _ = data_tx.send(Msg::Chunk(
                    data.iter()
                        .map(|s| (f32::from(*s) - 32768.0) / 32768.0)
                        .collect(),
                ));
            },
            on_error,
            None,
        ),
        other => return Err(format!("Unsupported microphone format: {other:?}")),
    }
    .map_err(|e| format!("Cannot open the microphone: {e}"))?;

    stream
        .play()
        .map_err(|e| format!("Cannot start the microphone: {e}"))?;

    Ok(Active {
        _stream: stream,
        resampler: Resampler::new(rate, SAMPLE_RATE),
        channels,
        samples: Vec::new(),
        detector: (silence_ms > 0)
            .then(|| SilenceDetector::new(silence_ms, NO_SPEECH_MS.max(silence_ms * 2))),
        started: false,
        session,
    })
}
