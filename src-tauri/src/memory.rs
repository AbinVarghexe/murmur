//! Persistent local storage for WhisperFlow-style user memory.
//!
//! The file stays in Murmur's data directory and is never sent to a server. The pure matching
//! rules live in `murmur-core::memory`; this module owns loading, saving and app integration.

use murmur_core::memory::{
    add_learning_example, add_or_update_entry, complete_learning_example, delete_correction,
    learn_correction, mark_learning_observation_unavailable, mark_learning_outcome,
    mark_transcript_corrected, personalize, record_transcript, LearningExampleResult,
    LearningResult, MemoryKind, PersonalizationResult, VoiceMemory,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const FILE_NAME: &str = "voice-memory.json";

fn now() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().to_string())
        .unwrap_or_else(|_| "0".into())
}

pub struct MemoryStore {
    path: PathBuf,
    data: VoiceMemory,
}

impl MemoryStore {
    pub fn load(data_dir: &Path) -> Self {
        let path = data_dir.join(FILE_NAME);
        let mut data = fs::read_to_string(&path)
            .ok()
            .and_then(|text| serde_json::from_str::<VoiceMemory>(&text).ok())
            .map(VoiceMemory::normalized)
            .unwrap_or_default();
        if data.entries.is_empty() {
            data.entries = murmur_core::memory::default_baseline_entries();
            let _ = fs::write(&path, serde_json::to_string_pretty(&data).unwrap_or_default());
        } else {
            let mut changed = false;
            for baseline in murmur_core::memory::default_baseline_entries() {
                if !data
                    .entries
                    .iter()
                    .any(|e| e.canonical.eq_ignore_ascii_case(&baseline.canonical))
                {
                    data.entries.push(baseline);
                    changed = true;
                }
            }
            if changed {
                let _ = fs::write(&path, serde_json::to_string_pretty(&data).unwrap_or_default());
            }
        }
        Self { path, data }
    }

    pub fn snapshot(&self) -> VoiceMemory {
        self.data.clone()
    }

    pub fn personalize(&mut self, input: &str, context: &str) -> PersonalizationResult {
        let result = personalize(&mut self.data, input, context, &now());
        if !result.replacements.is_empty() {
            let _ = self.save();
        }
        result
    }

    pub fn record_transcript(&mut self, raw: &str, personalized: &str, mode: &str) {
        record_transcript(&mut self.data, raw, personalized, mode, &now());
        let _ = self.save();
    }

    pub fn add_learning_example(
        &mut self,
        raw: &str,
        inserted: &str,
        audio_path: &str,
        applied_entry_ids: &[String],
    ) -> String {
        let id = add_learning_example(
            &mut self.data,
            raw,
            inserted,
            audio_path,
            applied_entry_ids,
            &now(),
        );
        let _ = self.save();
        id
    }

    pub fn complete_learning_example(
        &mut self,
        id: &str,
        observed: &str,
        context: &str,
    ) -> LearningExampleResult {
        let result = complete_learning_example(&mut self.data, id, observed, context, &now());
        if result.status != "missing" {
            let _ = self.save();
        }
        result
    }

    pub fn mark_learning_outcome(&mut self, id: &str, success: bool) -> bool {
        let changed = mark_learning_outcome(&mut self.data, id, success, &now());
        if changed {
            let _ = self.save();
        }
        changed
    }

    pub fn mark_learning_observation_unavailable(&mut self, id: &str) -> bool {
        let changed = mark_learning_observation_unavailable(&mut self.data, id, &now());
        if changed {
            let _ = self.save();
        }
        changed
    }

    pub fn learn_correction(
        &mut self,
        original: &str,
        corrected: &str,
        context: &str,
    ) -> LearningResult {
        let result = learn_correction(&mut self.data, original, corrected, context, &now());
        mark_transcript_corrected(&mut self.data, original, corrected);
        let _ = self.save();
        result
    }

    pub fn add_entry(
        &mut self,
        canonical: &str,
        aliases: &[String],
        kind: MemoryKind,
        id: Option<&str>,
    ) -> Option<String> {
        let result = add_or_update_entry(&mut self.data, canonical, aliases, kind, &now(), id);
        if result.is_some() {
            let _ = self.save();
        }
        result
    }

    pub fn delete_entry(&mut self, id: &str) -> bool {
        let before = self.data.entries.len();
        self.data.entries.retain(|entry| entry.id != id);
        if before != self.data.entries.len() {
            let _ = self.save();
            return true;
        }
        false
    }

    pub fn delete_correction(&mut self, id: &str) -> bool {
        if delete_correction(&mut self.data, id) {
            let _ = self.save();
            return true;
        }
        false
    }

    pub fn clear(&mut self) -> Result<(), String> {
        for example in &self.data.learning_examples {
            let path = self
                .path
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .join(&example.audio_path);
            let _ = fs::remove_file(path);
        }
        self.data = VoiceMemory::default();
        self.save()
    }

    fn save(&self) -> Result<(), String> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("Cannot create voice memory folder: {error}"))?;
        }
        let json = serde_json::to_string_pretty(&self.data).map_err(|error| error.to_string())?;
        let temporary = self.path.with_extension("json.tmp");
        fs::write(&temporary, json)
            .map_err(|error| format!("Cannot save voice memory: {error}"))?;
        fs::rename(&temporary, &self.path)
            .map_err(|error| format!("Cannot finish voice memory save: {error}"))
    }
}
