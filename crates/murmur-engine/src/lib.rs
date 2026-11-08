//! The offline speech engine: sherpa-onnx running NVIDIA's Parakeet TDT model on this computer.
//!
//! A thin, safe wrapper over the handful of sherpa-onnx C functions needed for non-streaming
//! transcription. All `unsafe` is confined to this file.

use murmur_core::audio_utils::pad_silence;
use murmur_core::chunk::split_at_silence;
use murmur_core::controller::SAMPLE_RATE;
use murmur_sherpa_sys as sys;
use std::ffi::{CStr, CString};
use std::path::Path;

/// Long recordings are split at pauses so no piece is longer than this.
const MAX_CHUNK_SECONDS: f32 = 30.0;
/// How far back from the limit to look for a quiet place to cut.
const SEARCH_SECONDS: f32 = 6.0;
/// Silence added around each piece. The first and last word are less often dropped when the
/// speech is not flush against the edge of the buffer.
const PAD_SECONDS: f32 = 0.4;
const FEATURE_DIM: i32 = 80;

pub struct Engine {
    recognizer: *const sys::SherpaOnnxOfflineRecognizer,
}

// SAFETY: a sherpa-onnx recognizer has no thread affinity; it only must not be used by two
// threads at once. `Engine` is only reachable through `&mut self`, and the app keeps it behind
// a Mutex, so use is always exclusive.
unsafe impl Send for Engine {}

impl Drop for Engine {
    fn drop(&mut self) {
        // SAFETY: `recognizer` came from `SherpaOnnxCreateOfflineRecognizer` and is destroyed once.
        unsafe { sys::SherpaOnnxDestroyOfflineRecognizer(self.recognizer) };
    }
}

fn thread_count() -> i32 {
    let cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(2);
    (cores / 2).clamp(1, 4) as i32
}

fn c_string(value: &str) -> Result<CString, String> {
    CString::new(value).map_err(|_| "A model path contains an invalid character".to_string())
}

impl Engine {
    /// Load the model. Takes a few seconds, so it is done ahead of the first recording.
    pub fn load(model_dir: &Path) -> Result<Engine, String> {
        let path = |name: &str| c_string(&model_dir.join(name).to_string_lossy());
        let encoder = path("encoder.int8.onnx")?;
        let decoder = path("decoder.int8.onnx")?;
        let joiner = path("joiner.int8.onnx")?;
        let tokens = path("tokens.txt")?;
        let provider = c_string("cpu")?;
        let model_type = c_string("nemo_transducer")?;
        let decoding = c_string("greedy_search")?;

        // SAFETY: the config is a plain C struct for which all-zero is valid (null pointers, 0,
        // 0.0). Only the fields below are set. Every pointer refers to a CString that outlives
        // the call, and the library copies what it needs during creation.
        let recognizer = unsafe {
            let mut config: sys::SherpaOnnxOfflineRecognizerConfig = std::mem::zeroed();
            config.feat_config.sample_rate = SAMPLE_RATE as i32;
            config.feat_config.feature_dim = FEATURE_DIM;
            config.model_config.transducer.encoder = encoder.as_ptr();
            config.model_config.transducer.decoder = decoder.as_ptr();
            config.model_config.transducer.joiner = joiner.as_ptr();
            config.model_config.tokens = tokens.as_ptr();
            config.model_config.num_threads = thread_count();
            config.model_config.debug = 0;
            config.model_config.provider = provider.as_ptr();
            config.model_config.model_type = model_type.as_ptr();
            config.decoding_method = decoding.as_ptr();
            sys::SherpaOnnxCreateOfflineRecognizer(&config)
        };
        if recognizer.is_null() {
            return Err("Cannot load the speech model. Reinstall it from Settings.".into());
        }
        Ok(Engine { recognizer })
    }

    fn decode(&mut self, samples: &[f32]) -> String {
        // SAFETY: `recognizer` is valid for the life of `self`. The stream and result are
        // created and destroyed here, and `samples` outlives the calls that read it. The text
        // pointer is read before the result is destroyed.
        unsafe {
            let stream = sys::SherpaOnnxCreateOfflineStream(self.recognizer);
            if stream.is_null() {
                return String::new();
            }
            sys::SherpaOnnxAcceptWaveformOffline(
                stream,
                SAMPLE_RATE as i32,
                samples.as_ptr(),
                samples.len() as i32,
            );
            sys::SherpaOnnxDecodeOfflineStream(self.recognizer, stream);
            let result = sys::SherpaOnnxGetOfflineStreamResult(stream);
            let text = if result.is_null() || (*result).text.is_null() {
                String::new()
            } else {
                CStr::from_ptr((*result).text)
                    .to_string_lossy()
                    .into_owned()
            };
            if !result.is_null() {
                sys::SherpaOnnxDestroyOfflineRecognizerResult(result);
            }
            sys::SherpaOnnxDestroyOfflineStream(stream);
            text
        }
    }

    /// Transcribe 16 kHz mono audio of any length.
    pub fn transcribe(&mut self, samples: &[f32]) -> String {
        let mut parts: Vec<String> = Vec::new();
        for chunk in split_at_silence(samples, SAMPLE_RATE, MAX_CHUNK_SECONDS, SEARCH_SECONDS) {
            let padded = pad_silence(chunk, SAMPLE_RATE, PAD_SECONDS);
            let text = self.decode(&padded);
            let text = text.trim();
            if !text.is_empty() {
                parts.push(text.to_string());
            }
        }
        parts.join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_model_is_an_error_not_a_crash() {
        let missing = std::env::temp_dir().join("murmur-no-such-model");
        assert!(Engine::load(&missing).is_err());
    }

    #[test]
    fn model_paths_with_nul_bytes_are_rejected() {
        assert!(c_string("bad\0path").is_err());
        assert!(c_string("fine").is_ok());
    }
}
