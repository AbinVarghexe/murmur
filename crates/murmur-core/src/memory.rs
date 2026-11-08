//! User-specific voice memory and conservative transcript personalization.
//!
//! The speech model remains general-purpose. This module ranks locally learned candidates around
//! the model output and only applies a replacement when the learned evidence is strong enough.
//! Persistence is deliberately kept outside this crate so the rules remain easy to test.

use serde::{Deserialize, Serialize};

const CURRENT_SCHEMA_VERSION: u32 = 2;
const MAX_CONTEXTS: usize = 8;
const MAX_RECENT_TRANSCRIPTS: usize = 50;
const MAX_LEARNING_EXAMPLES: usize = 200;
const MIN_EXACT_SCORE: f32 = 0.76;
const MIN_FUZZY_SCORE: f32 = 0.88;
const MIN_FUZZY_CONFIDENCE: f32 = 0.82;
const MIN_CONFLICT_MARGIN: f32 = 0.08;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryKind {
    Person,
    Vocabulary,
    Phrase,
}

impl Default for MemoryKind {
    fn default() -> Self {
        Self::Vocabulary
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryEntry {
    pub id: String,
    #[serde(default)]
    pub kind: MemoryKind,
    pub canonical: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub contexts: Vec<String>,
    #[serde(default)]
    pub frequency: u32,
    #[serde(default)]
    pub confidence: f32,
    /// The user explicitly supplied or confirmed this mapping. Explicit knowledge can be
    /// applied immediately for exact matches; inferred/fuzzy knowledge still needs evidence.
    #[serde(default)]
    pub explicit: bool,
    /// Becomes true after repeated confirmed evidence; manual entries are promoted immediately.
    #[serde(default)]
    pub promoted: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LearnedCorrection {
    pub id: String,
    pub original: String,
    pub corrected: String,
    #[serde(default)]
    pub context: String,
    #[serde(default)]
    pub frequency: u32,
    #[serde(default)]
    pub confidence: f32,
    /// Number of future dictations where this correction was applied and accepted without
    /// an edit. This is separate from `frequency`, which counts user-provided corrections.
    #[serde(default)]
    pub successes: u32,
    /// Number of future dictations where this correction was applied and then edited again.
    #[serde(default)]
    pub failures: u32,
    /// A correction is promoted after it has enough independent evidence to be treated as a
    /// reliable personal rule.
    #[serde(default)]
    pub promoted: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptRecord {
    pub id: String,
    pub raw: String,
    pub personalized: String,
    #[serde(default)]
    pub corrected: Option<String>,
    #[serde(default)]
    pub mode: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LearningExample {
    pub id: String,
    /// Path relative to Murmur's data directory. The audio is kept outside the JSON index.
    pub audio_path: String,
    pub raw: String,
    pub inserted: String,
    #[serde(default)]
    pub observed: Option<String>,
    #[serde(default)]
    pub corrected: Option<String>,
    #[serde(default)]
    pub correction_id: Option<String>,
    /// Personal-memory entries that were applied before this text was inserted. Keeping this
    /// provenance lets the post-insertion observer attribute success or failure to the exact
    /// correction instead of only updating an anonymous example counter.
    #[serde(default)]
    pub applied_entry_ids: Vec<String>,
    /// pending → unchanged, edited, or learned.
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub successes: u32,
    #[serde(default)]
    pub failures: u32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct VoiceMemory {
    pub schema_version: u32,
    pub entries: Vec<MemoryEntry>,
    pub corrections: Vec<LearnedCorrection>,
    pub recent_transcripts: Vec<TranscriptRecord>,
    #[serde(default)]
    pub learning_examples: Vec<LearningExample>,
}

pub fn default_baseline_entries() -> Vec<MemoryEntry> {
    vec![
        MemoryEntry {
            id: "memory-epinogees".into(),
            kind: MemoryKind::Person,
            canonical: "epinogees".into(),
            aliases: vec![
                "epic geese".into(),
                "apin ogees".into(),
                "app in ogees".into(),
                "epinogis".into(),
                "epinogee".into(),
                "apinogees".into(),
                "epicgeese".into(),
                "api nogees".into(),
                "app in a geese".into(),
            ],
            contexts: vec!["personal".into(), "identity".into()],
            frequency: 10,
            confidence: 0.98,
            explicit: true,
            promoted: true,
            created_at: "0".into(),
            updated_at: "0".into(),
        },
        MemoryEntry {
            id: "memory-abin-varghese".into(),
            kind: MemoryKind::Person,
            canonical: "Abin Varghese".into(),
            aliases: vec![
                "abin vargis".into(),
                "eben vergis".into(),
                "eben vargis".into(),
                "ebin varghese".into(),
                "a bean varghese".into(),
                "abin vergis".into(),
            ],
            contexts: vec!["name".into(), "author".into()],
            frequency: 10,
            confidence: 0.98,
            explicit: true,
            promoted: true,
            created_at: "0".into(),
            updated_at: "0".into(),
        },
        MemoryEntry {
            id: "memory-antigravity".into(),
            kind: MemoryKind::Vocabulary,
            canonical: "Antigravity".into(),
            aliases: vec![
                "anti gravity".into(),
                "anti-gravity".into(),
                "antigravity ide".into(),
                "anti gravity ide".into(),
            ],
            contexts: vec!["editor".into(), "ide".into(), "coding".into()],
            frequency: 12,
            confidence: 0.98,
            explicit: true,
            promoted: true,
            created_at: "0".into(),
            updated_at: "0".into(),
        },
        MemoryEntry {
            id: "memory-chatgpt".into(),
            kind: MemoryKind::Vocabulary,
            canonical: "ChatGPT".into(),
            aliases: vec![
                "chat gpt".into(),
                "chat-gpt".into(),
                "chatgpt".into(),
                "chat gbt".into(),
                "gpt".into(),
            ],
            contexts: vec!["ai".into(), "tool".into()],
            frequency: 12,
            confidence: 0.98,
            explicit: true,
            promoted: true,
            created_at: "0".into(),
            updated_at: "0".into(),
        },
        MemoryEntry {
            id: "memory-figma".into(),
            kind: MemoryKind::Vocabulary,
            canonical: "Figma".into(),
            aliases: vec![
                "fig ma".into(),
                "figma".into(),
                "figmer".into(),
                "pig ma".into(),
                "fig nah".into(),
            ],
            contexts: vec!["design".into(), "ui".into()],
            frequency: 10,
            confidence: 0.98,
            explicit: true,
            promoted: true,
            created_at: "0".into(),
            updated_at: "0".into(),
        },
    ]
}

impl Default for VoiceMemory {
    fn default() -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            entries: Vec::new(),
            corrections: Vec::new(),
            recent_transcripts: Vec::new(),
            learning_examples: Vec::new(),
        }
    }
}

impl VoiceMemory {
    pub fn normalized(mut self) -> Self {
        self.schema_version = CURRENT_SCHEMA_VERSION;
        self.entries
            .retain(|entry| !entry.canonical.trim().is_empty());
        self.corrections.retain(|correction| {
            !correction.original.trim().is_empty() && !correction.corrected.trim().is_empty()
        });
        self.entries.truncate(500);
        self.corrections.truncate(500);
        self.recent_transcripts.truncate(MAX_RECENT_TRANSCRIPTS);
        self.learning_examples
            .retain(|example| !example.id.trim().is_empty() && !example.inserted.trim().is_empty());
        self.learning_examples.truncate(MAX_LEARNING_EXAMPLES);

        // Migrate corrections created before explicit provenance was stored. A correction
        // submitted by the user is explicit knowledge even when it came from an older file.
        for entry in &mut self.entries {
            if let Some(correction) = self.corrections.iter().find(|correction| {
                key(&entry.canonical) == key(&correction.corrected)
                    && entry
                        .aliases
                        .iter()
                        .any(|alias| key(alias) == key(&correction.original))
            }) {
                entry.explicit = true;
                entry.promoted = correction.frequency >= 2;
            }
        }
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Replacement {
    pub from: String,
    pub to: String,
    pub score: f32,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonalizationResult {
    pub text: String,
    pub replacements: Vec<Replacement>,
    /// IDs of memory entries used during this pass. These are carried into the learning example
    /// so later editor observations can update the right correction's reliability.
    #[serde(default)]
    pub applied_entry_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LearningResult {
    pub learned: Vec<LearnedCorrection>,
    pub entry_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LearningExampleResult {
    pub id: String,
    pub status: String,
    pub corrected: Option<String>,
    pub learned: LearningResult,
}

#[derive(Debug, Clone)]
struct Candidate {
    start: usize,
    end: usize,
    alias: String,
    canonical: String,
    score: f32,
    entry_index: usize,
    reason: String,
}

fn clean(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn key(value: &str) -> String {
    clean(value)
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_lowercase().next().unwrap_or(c)
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn compact_key(value: &str) -> String {
    value
        .chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(|c| c.to_lowercase())
        .collect()
}

fn word_char(c: char) -> bool {
    c.is_alphanumeric()
}

fn context_similarity(context: &str, stored: &[String]) -> f32 {
    let context_words: Vec<String> = key(context)
        .split_whitespace()
        .map(str::to_string)
        .collect();
    if context_words.is_empty() || stored.is_empty() {
        return 0.0;
    }
    let mut best = 0.0_f32;
    for sample in stored {
        let sample_words: Vec<String> =
            key(sample).split_whitespace().map(str::to_string).collect();
        let shared = context_words
            .iter()
            .filter(|word| sample_words.contains(word))
            .count();
        let denom = context_words.len().max(sample_words.len()) as f32;
        best = best.max(shared as f32 / denom.max(1.0));
    }
    best
}

fn frequency_bonus(frequency: u32) -> f32 {
    (frequency as f32 / 10.0).min(1.0) * 0.08
}

fn exact_score(entry: &MemoryEntry, context: &str) -> f32 {
    let confidence = entry.confidence.clamp(0.0, 1.0);
    let context_bonus = context_similarity(context, &entry.contexts) * 0.12;
    if entry.explicit {
        // User-provided mappings are trusted for exact matches immediately. Their fuzzy
        // candidates still have to clear MIN_FUZZY_SCORE, and conflicts still need a margin.
        (0.64 + confidence * 0.20 + frequency_bonus(entry.frequency) + context_bonus).min(1.0)
    } else {
        (0.48 + confidence * 0.32 + frequency_bonus(entry.frequency) + context_bonus).min(1.0)
    }
}

fn levenshtein_similarity(left: &str, right: &str) -> f32 {
    let a: Vec<char> = left.chars().collect();
    let b: Vec<char> = right.chars().collect();
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let mut previous: Vec<usize> = (0..=b.len()).collect();
    let mut current = vec![0; b.len() + 1];
    for (i, left_char) in a.iter().enumerate() {
        current[0] = i + 1;
        for (j, right_char) in b.iter().enumerate() {
            let cost = usize::from(left_char != right_char);
            current[j + 1] = (current[j] + 1)
                .min(previous[j + 1] + 1)
                .min(previous[j] + cost);
        }
        std::mem::swap(&mut previous, &mut current);
    }
    1.0 - previous[b.len()] as f32 / a.len().max(b.len()) as f32
}

fn phonetic_key(word: &str) -> String {
    let lower = word.to_lowercase();
    let mut result = String::new();
    let mut previous = ' ';
    for (index, character) in lower.chars().enumerate() {
        if !character.is_ascii_alphabetic() {
            continue;
        }
        if index == 0 {
            result.push(character);
            previous = character;
            continue;
        }
        let mapped = match character {
            'b' | 'f' | 'p' | 'v' => '1',
            'c' | 'g' | 'j' | 'k' | 'q' | 's' | 'x' | 'z' => '2',
            'd' | 't' => '3',
            'l' => '4',
            'm' | 'n' => '5',
            'r' => '6',
            _ => '0',
        };
        if mapped != '0' && mapped != previous {
            result.push(mapped);
        }
        previous = mapped;
    }
    result.chars().take(5).collect()
}

fn phonetic_similarity(left: &str, right: &str) -> f32 {
    let left_key = key(left);
    let right_key = key(right);
    let left_words: Vec<&str> = left_key.split_whitespace().collect();
    let right_words: Vec<&str> = right_key.split_whitespace().collect();
    if left_words.len() != right_words.len() || left_words.is_empty() {
        return 0.0;
    }
    left_words
        .iter()
        .zip(right_words.iter())
        .map(|(a, b)| f32::from(phonetic_key(a) == phonetic_key(b)))
        .sum::<f32>()
        / left_words.len() as f32
}

fn word_spans(input: &str) -> Vec<(usize, usize)> {
    let mut spans = Vec::new();
    let mut start = None;
    for (index, character) in input.char_indices() {
        if word_char(character) {
            if start.is_none() {
                start = Some(index);
            }
        } else if let Some(word_start) = start.take() {
            spans.push((word_start, index));
        }
    }
    if let Some(word_start) = start {
        spans.push((word_start, input.len()));
    }
    spans
}

/// Find phrase matches after normalizing case, punctuation, and whitespace. The old matcher
/// compared raw substrings, so an alias such as `Eben Vergis.` could not match `Eben Vergis`.
fn normalized_matches(input: &str, needle: &str) -> Vec<(usize, usize)> {
    let needle_key = key(needle);
    let needle_words = needle_key.split_whitespace().count();
    let spans = word_spans(input);
    if needle_words == 0 || spans.len() < needle_words {
        return Vec::new();
    }
    let mut matches = Vec::new();
    for window in spans.windows(needle_words) {
        let start = window[0].0;
        let end = window.last().map(|part| part.1).unwrap_or(start);
        if key(&input[start..end]) == needle_key {
            matches.push((start, end));
        }
    }
    matches
}

fn candidate_for_alias(
    entry_index: usize,
    entry: &MemoryEntry,
    alias: &str,
    input: &str,
    context: &str,
) -> Vec<Candidate> {
    let alias_clean = clean(alias);
    let exact = normalized_matches(input, &alias_clean);
    if !exact.is_empty() {
        if !entry.explicit && entry.frequency < 2 && entry.confidence < 0.8 {
            return Vec::new();
        }
        let score = exact_score(entry, context);
        return exact
            .into_iter()
            .map(|(start, end)| Candidate {
                start,
                end,
                alias: alias_clean.clone(),
                canonical: entry.canonical.clone(),
                score,
                entry_index,
                reason: "exact learned alias".into(),
            })
            .collect();
    }

    let alias_words: Vec<&str> = alias_clean.split_whitespace().collect();
    if alias_words.is_empty()
        || (!entry.explicit && (entry.confidence < MIN_FUZZY_CONFIDENCE || entry.frequency < 2))
    {
        return Vec::new();
    }

    let mut input_words: Vec<(usize, usize, &str)> = Vec::new();
    let mut word_start: Option<usize> = None;
    for (index, character) in input.char_indices() {
        if word_start.is_none() && word_char(character) {
            word_start = Some(index);
        } else if word_start.is_some() && !word_char(character) {
            let start = word_start.take().unwrap_or(index);
            input_words.push((start, index, &input[start..index]));
        }
    }
    if let Some(start) = word_start {
        input_words.push((start, input.len(), &input[start..]));
    }
    if input_words.is_empty() {
        return Vec::new();
    }

    let alias_compact = compact_key(&alias_clean);
    let canonical_compact = compact_key(&entry.canonical);
    let mut results = Vec::new();

    // Check sliding window sizes around alias_words.len()
    let min_w = alias_words.len().saturating_sub(1).max(1);
    let max_w = (alias_words.len() + 1).min(input_words.len()).max(min_w);

    for w_len in min_w..=max_w {
        if w_len > input_words.len() {
            continue;
        }
        for window in input_words.windows(w_len) {
            let start = window[0].0;
            let end = window.last().map(|part| part.1).unwrap_or(start);
            let phrase = window
                .iter()
                .map(|part| part.2)
                .collect::<Vec<_>>()
                .join(" ");
            let phrase_compact = compact_key(&phrase);

            if !alias_compact.is_empty() && (phrase_compact == alias_compact || phrase_compact == canonical_compact) {
                let score = exact_score(entry, context).max(0.92);
                results.push(Candidate {
                    start,
                    end,
                    alias: phrase,
                    canonical: entry.canonical.clone(),
                    score,
                    entry_index,
                    reason: "compact whitespace-normalized alias".into(),
                });
                continue;
            }

            if w_len == alias_words.len() {
                let text_similarity = levenshtein_similarity(&key(&phrase), &key(&alias_clean));
                let sound_similarity = phonetic_similarity(&phrase, &alias_clean);
                let score =
                    (text_similarity * 0.55 + sound_similarity * 0.25 + exact_score(entry, context) * 0.20)
                        .min(1.0);
                let threshold = if entry.explicit {
                    // The user explicitly supplied this alias, so a strong phonetic variant such as
                    // "Eben Varges" for "Eben Vergis" can adapt live without waiting for retraining.
                    0.84
                } else {
                    MIN_FUZZY_SCORE
                };
                if score >= threshold {
                    results.push(Candidate {
                        start,
                        end,
                        alias: phrase,
                        canonical: entry.canonical.clone(),
                        score,
                        entry_index,
                        reason: "fuzzy and phonetic learned alias".into(),
                    });
                }
            }
        }
    }
    results
}

fn replacement_candidates(memory: &VoiceMemory, input: &str, context: &str) -> Vec<Candidate> {
    let mut candidates = Vec::new();
    for (entry_index, entry) in memory.entries.iter().enumerate() {
        // Evaluate the canonical name itself so variations in casing/spacing/hyphenation match
        candidates.extend(candidate_for_alias(
            entry_index,
            entry,
            &entry.canonical,
            input,
            context,
        ));
        for alias in &entry.aliases {
            if key(alias) != key(&entry.canonical) {
                candidates.extend(candidate_for_alias(
                    entry_index,
                    entry,
                    alias,
                    input,
                    context,
                ));
            }
        }
    }
    candidates
}

/// Apply learned memory with a score threshold and a conflict margin.
pub fn personalize(
    memory: &mut VoiceMemory,
    input: &str,
    context: &str,
    now: &str,
) -> PersonalizationResult {
    let mut candidates = replacement_candidates(memory, input, context);
    candidates.sort_by(|left, right| {
        left.start
            .cmp(&right.start)
            .then_with(|| right.score.total_cmp(&left.score))
            .then_with(|| (right.end - right.start).cmp(&(left.end - left.start)))
    });

    let mut accepted: Vec<Candidate> = Vec::new();
    for candidate in candidates {
        if candidate.score < MIN_EXACT_SCORE {
            continue;
        }
        let conflicting = replacement_candidates(memory, input, context)
            .into_iter()
            .filter(|other| {
                other.start == candidate.start
                    && other.end == candidate.end
                    && other.canonical != candidate.canonical
            })
            .map(|other| other.score)
            .max_by(f32::total_cmp);
        if conflicting.is_some_and(|score| candidate.score - score < MIN_CONFLICT_MARGIN) {
            continue;
        }
        if accepted
            .iter()
            .any(|other| candidate.start < other.end && other.start < candidate.end)
        {
            continue;
        }
        accepted.push(candidate);
    }
    accepted.sort_by_key(|candidate| candidate.start);

    let mut output = String::with_capacity(input.len());
    let mut cursor = 0;
    let mut replacements = Vec::new();
    let mut applied_entry_ids = Vec::new();
    for candidate in &accepted {
        output.push_str(&input[cursor..candidate.start]);
        output.push_str(&candidate.canonical);
        cursor = candidate.end;
        replacements.push(Replacement {
            from: candidate.alias.clone(),
            to: candidate.canonical.clone(),
            score: candidate.score,
            reason: candidate.reason.clone(),
        });
        if let Some(entry) = memory.entries.get_mut(candidate.entry_index) {
            entry.frequency = entry.frequency.saturating_add(1);
            entry.updated_at = now.to_string();
            if !applied_entry_ids.iter().any(|id| id == &entry.id) {
                applied_entry_ids.push(entry.id.clone());
            }
        }
    }
    output.push_str(&input[cursor..]);
    PersonalizationResult {
        text: output,
        replacements,
        applied_entry_ids,
    }
}

fn changed_segment(original: &str, corrected: &str) -> Option<(String, String)> {
    let original_clean = clean(original);
    let corrected_clean = clean(corrected);
    let original_words: Vec<&str> = original_clean.split_whitespace().collect();
    let corrected_words: Vec<&str> = corrected_clean.split_whitespace().collect();
    if original_words.is_empty() || corrected_words.is_empty() {
        return None;
    }
    let mut start = 0;
    while start < original_words.len()
        && start < corrected_words.len()
        && key(original_words[start]) == key(corrected_words[start])
    {
        start += 1;
    }
    let mut original_end = original_words.len();
    let mut corrected_end = corrected_words.len();
    while original_end > start
        && corrected_end > start
        && key(original_words[original_end - 1]) == key(corrected_words[corrected_end - 1])
    {
        original_end -= 1;
        corrected_end -= 1;
    }
    let old = original_words[start..original_end].join(" ");
    let new = corrected_words[start..corrected_end].join(" ");
    (old.is_empty() || new.is_empty())
        .then_some((clean(original), clean(corrected)))
        .or_else(|| Some((old, new)))
}

fn upsert_entry(
    memory: &mut VoiceMemory,
    canonical: &str,
    alias: &str,
    kind: MemoryKind,
    context: &str,
    now: &str,
    id: Option<&str>,
    explicit: bool,
) -> String {
    let canonical = clean(canonical);
    let alias = clean(alias);
    let entry_index = id
        .and_then(|target| memory.entries.iter().position(|entry| entry.id == target))
        .or_else(|| {
            memory
                .entries
                .iter()
                .position(|entry| key(&entry.canonical) == key(&canonical))
        });
    if let Some(index) = entry_index {
        let entry = &mut memory.entries[index];
        entry.canonical = canonical;
        entry.kind = kind;
        if !alias.is_empty()
            && key(&entry.canonical) != key(&alias)
            && !entry.aliases.iter().any(|item| key(item) == key(&alias))
        {
            entry.aliases.push(alias);
        }
        if !context.is_empty() && !entry.contexts.iter().any(|item| key(item) == key(context)) {
            entry.contexts.push(clean(context));
            entry.contexts.truncate(MAX_CONTEXTS);
        }
        entry.frequency = entry.frequency.max(1);
        entry.explicit |= explicit;
        if explicit {
            entry.promoted = false;
        }
        entry.updated_at = now.to_string();
        return entry.id.clone();
    }

    let id = id
        .map(str::to_string)
        .unwrap_or_else(|| format!("memory-{}", memory.entries.len() + 1));
    memory.entries.push(MemoryEntry {
        id: id.clone(),
        kind,
        canonical,
        aliases: if alias.is_empty() {
            Vec::new()
        } else {
            vec![alias]
        },
        contexts: if context.is_empty() {
            Vec::new()
        } else {
            vec![clean(context)]
        },
        frequency: 1,
        confidence: 0.62,
        explicit,
        promoted: false,
        created_at: now.to_string(),
        updated_at: now.to_string(),
    });
    id
}

/// Learn the changed phrase, keeping contradictory corrections as separate candidates.
pub fn learn_correction(
    memory: &mut VoiceMemory,
    original: &str,
    corrected: &str,
    context: &str,
    now: &str,
) -> LearningResult {
    let Some((spoken, canonical)) = changed_segment(original, corrected) else {
        return LearningResult {
            learned: Vec::new(),
            entry_ids: Vec::new(),
        };
    };
    if key(&spoken).is_empty() || key(&spoken) == key(&canonical) {
        return LearningResult {
            learned: Vec::new(),
            entry_ids: Vec::new(),
        };
    }

    let correction_index = memory.corrections.iter().position(|item| {
        key(&item.original) == key(&spoken) && key(&item.corrected) == key(&canonical)
    });
    let correction = if let Some(index) = correction_index {
        let item = &mut memory.corrections[index];
        item.frequency = item.frequency.saturating_add(1);
        item.confidence = (item.confidence + 0.08).min(0.99);
        if !context.is_empty() {
            item.context = clean(context);
        }
        item.updated_at = now.to_string();
        item.clone()
    } else {
        let item = LearnedCorrection {
            id: format!("correction-{}", memory.corrections.len() + 1),
            original: clean(&spoken),
            corrected: clean(&canonical),
            context: clean(context),
            frequency: 1,
            confidence: 0.62,
            successes: 0,
            failures: 0,
            promoted: false,
            created_at: now.to_string(),
            updated_at: now.to_string(),
        };
        memory.corrections.insert(0, item.clone());
        item
    };

    // A contradictory correction remains visible, but its candidate only wins with a margin.
    for item in memory.corrections.iter_mut().filter(|item| {
        key(&item.original) == key(&spoken) && key(&item.corrected) != key(&canonical)
    }) {
        item.confidence = (item.confidence - 0.04).max(0.35);
        item.updated_at = now.to_string();
    }

    let entry_id = upsert_entry(
        memory,
        &correction.corrected,
        &correction.original,
        if correction.corrected.split_whitespace().count() >= 2 {
            MemoryKind::Person
        } else {
            MemoryKind::Vocabulary
        },
        context,
        now,
        None,
        true,
    );
    if let Some(entry) = memory.entries.iter_mut().find(|entry| entry.id == entry_id) {
        entry.frequency = correction.frequency;
        entry.confidence = correction.confidence;
        entry.promoted = correction.frequency >= 2 || correction.successes >= 2;
    }
    LearningResult {
        learned: vec![correction],
        entry_ids: vec![entry_id],
    }
}

pub fn add_or_update_entry(
    memory: &mut VoiceMemory,
    canonical: &str,
    aliases: &[String],
    kind: MemoryKind,
    now: &str,
    id: Option<&str>,
) -> Option<String> {
    if clean(canonical).is_empty() {
        return None;
    }
    let primary_alias = aliases.first().map(String::as_str).unwrap_or_default();
    let entry_id = upsert_entry(memory, canonical, primary_alias, kind, "", now, id, true);
    if let Some(entry) = memory.entries.iter_mut().find(|entry| entry.id == entry_id) {
        for alias in aliases
            .iter()
            .map(|alias| clean(alias))
            .filter(|alias| !alias.is_empty())
        {
            if key(&alias) != key(&entry.canonical)
                && !entry.aliases.iter().any(|item| key(item) == key(&alias))
            {
                entry.aliases.push(alias);
            }
        }
        entry.confidence = entry.confidence.max(0.8);
        entry.promoted = true;
        entry.updated_at = now.to_string();
    }
    Some(entry_id)
}

pub fn record_transcript(
    memory: &mut VoiceMemory,
    raw: &str,
    personalized: &str,
    mode: &str,
    now: &str,
) {
    memory.recent_transcripts.insert(
        0,
        TranscriptRecord {
            id: format!("transcript-{}", now),
            raw: clean(raw),
            personalized: clean(personalized),
            corrected: None,
            mode: mode.to_string(),
            timestamp: now.to_string(),
        },
    );
    memory.recent_transcripts.truncate(MAX_RECENT_TRANSCRIPTS);
}

pub fn add_learning_example(
    memory: &mut VoiceMemory,
    raw: &str,
    inserted: &str,
    audio_path: &str,
    applied_entry_ids: &[String],
    now: &str,
) -> String {
    let id = format!("learning-{}", now);
    memory.learning_examples.insert(
        0,
        LearningExample {
            id: id.clone(),
            audio_path: audio_path.to_string(),
            raw: clean(raw),
            inserted: clean(inserted),
            observed: None,
            corrected: None,
            correction_id: None,
            applied_entry_ids: applied_entry_ids.to_vec(),
            status: "pending".into(),
            successes: 0,
            failures: 0,
            created_at: now.to_string(),
            updated_at: now.to_string(),
        },
    );
    memory.learning_examples.truncate(MAX_LEARNING_EXAMPLES);
    id
}

pub fn complete_learning_example(
    memory: &mut VoiceMemory,
    id: &str,
    observed: &str,
    context: &str,
    now: &str,
) -> LearningExampleResult {
    let Some(index) = memory
        .learning_examples
        .iter()
        .position(|example| example.id == id)
    else {
        return LearningExampleResult {
            id: id.to_string(),
            status: "missing".into(),
            corrected: None,
            learned: LearningResult {
                learned: Vec::new(),
                entry_ids: Vec::new(),
            },
        };
    };
    let observed = clean(observed);
    let inserted = memory.learning_examples[index].inserted.clone();
    let applied_entry_ids = memory.learning_examples[index].applied_entry_ids.clone();
    if key(&inserted) == key(&observed) {
        let status = {
            let example = &mut memory.learning_examples[index];
            example.observed = Some(observed);
            example.status = "unchanged".into();
            example.successes = example.successes.saturating_add(1);
            example.updated_at = now.to_string();
            example.status.clone()
        };
        update_applied_correction_outcomes(memory, &applied_entry_ids, true, now);
        return LearningExampleResult {
            id: id.to_string(),
            status,
            corrected: None,
            learned: LearningResult {
                learned: Vec::new(),
                entry_ids: Vec::new(),
            },
        };
    }

    update_applied_correction_outcomes(memory, &applied_entry_ids, false, now);
    let learned = learn_correction(memory, &inserted, &observed, context, now);
    mark_transcript_corrected(memory, &inserted, &observed);
    let example = &mut memory.learning_examples[index];
    example.observed = Some(observed.clone());
    example.corrected = Some(observed.clone());
    example.correction_id = learned.learned.first().map(|item| item.id.clone());
    example.status = if learned.learned.is_empty() {
        "edited"
    } else {
        "learned"
    }
    .into();
    // An edit is evidence that the just-inserted transcript was not accepted. Future examples
    // can then record whether the learned correction was accepted without an edit.
    example.failures = example.failures.saturating_add(1);
    example.updated_at = now.to_string();
    LearningExampleResult {
        id: id.to_string(),
        status: example.status.clone(),
        corrected: Some(observed),
        learned,
    }
}

pub fn mark_learning_outcome(memory: &mut VoiceMemory, id: &str, success: bool, now: &str) -> bool {
    let applied_entry_ids = memory
        .learning_examples
        .iter()
        .find(|example| example.id == id)
        .map(|example| example.applied_entry_ids.clone());
    let Some(example) = memory
        .learning_examples
        .iter_mut()
        .find(|example| example.id == id)
    else {
        return false;
    };
    if success {
        example.successes = example.successes.saturating_add(1);
        example.status = "unchanged".into();
    } else {
        example.failures = example.failures.saturating_add(1);
        example.status = "failed".into();
    }
    example.updated_at = now.to_string();
    if let Some(applied_entry_ids) = applied_entry_ids {
        update_applied_correction_outcomes(memory, &applied_entry_ids, success, now);
    }
    true
}

/// Mark that the focused editor could not be observed. This is intentionally different from a
/// failed correction: the application may simply not expose UI Automation text APIs.
pub fn mark_learning_observation_unavailable(
    memory: &mut VoiceMemory,
    id: &str,
    now: &str,
) -> bool {
    let Some(example) = memory
        .learning_examples
        .iter_mut()
        .find(|example| example.id == id)
    else {
        return false;
    };
    example.status = "unobserved".into();
    example.updated_at = now.to_string();
    true
}

/// Attribute an observed post-insertion outcome to the exact memory entries that were applied.
/// This is the missing link between “the text was personalized” and “that personalization was
/// accepted by the user.” It lets reliable corrections promote while repeated failures lower
/// confidence without retraining the speech model.
fn update_applied_correction_outcomes(
    memory: &mut VoiceMemory,
    applied_entry_ids: &[String],
    success: bool,
    now: &str,
) {
    if applied_entry_ids.is_empty() {
        return;
    }

    let linked_entries: Vec<(String, Vec<String>)> = memory
        .entries
        .iter()
        .filter(|entry| applied_entry_ids.iter().any(|id| id == &entry.id))
        .map(|entry| (entry.canonical.clone(), entry.aliases.clone()))
        .collect();

    let mut promoted_canonicals = Vec::new();
    for correction in &mut memory.corrections {
        let linked = linked_entries.iter().any(|(canonical, aliases)| {
            key(canonical) == key(&correction.corrected)
                && aliases
                    .iter()
                    .any(|alias| key(alias) == key(&correction.original))
        });
        if !linked {
            continue;
        }
        if success {
            correction.successes = correction.successes.saturating_add(1);
            correction.confidence = (correction.confidence + 0.06).min(0.99);
        } else {
            correction.failures = correction.failures.saturating_add(1);
            correction.confidence = (correction.confidence - 0.08).max(0.35);
        }
        correction.promoted = correction.frequency >= 2 || correction.successes >= 2;
        if correction.promoted {
            promoted_canonicals.push(correction.corrected.clone());
        }
        correction.updated_at = now.to_string();
    }

    for entry in &mut memory.entries {
        if !applied_entry_ids.iter().any(|id| id == &entry.id) {
            continue;
        }
        if success {
            entry.confidence = (entry.confidence + 0.03).min(0.99);
        } else {
            entry.confidence = (entry.confidence - 0.04).max(0.35);
        }
        entry.promoted = entry.promoted
            || entry.confidence >= 0.9
            || promoted_canonicals
                .iter()
                .any(|canonical| key(canonical) == key(&entry.canonical));
        entry.updated_at = now.to_string();
    }
}

pub fn mark_transcript_corrected(memory: &mut VoiceMemory, original: &str, corrected: &str) {
    if let Some(record) = memory.recent_transcripts.iter_mut().find(|record| {
        key(&record.personalized) == key(original) || key(&record.raw) == key(original)
    }) {
        record.corrected = Some(clean(corrected));
    }
}

pub fn delete_correction(memory: &mut VoiceMemory, id: &str) -> bool {
    let Some(index) = memory
        .corrections
        .iter()
        .position(|correction| correction.id == id)
    else {
        return false;
    };
    let removed = memory.corrections.remove(index);
    let still_exists = memory.corrections.iter().any(|correction| {
        key(&correction.original) == key(&removed.original)
            && key(&correction.corrected) == key(&removed.corrected)
    });
    if !still_exists {
        for entry in &mut memory.entries {
            if key(&entry.canonical) == key(&removed.corrected) {
                entry
                    .aliases
                    .retain(|alias| key(alias) != key(&removed.original));
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn memory() -> VoiceMemory {
        VoiceMemory::default()
    }

    #[test]
    fn an_explicit_correction_applies_on_the_next_exact_match() {
        let mut value = memory();
        learn_correction(
            &mut value,
            "I sent it to Eben Vargis",
            "I sent it to Abin Varghese",
            "send report",
            "1",
        );
        let result = personalize(&mut value, "I sent it to Eben Vargis", "send report", "2");
        assert_eq!(result.text, "I sent it to Abin Varghese");
        assert_eq!(result.replacements.len(), 1);
    }

    #[test]
    fn explicit_entries_ignore_terminal_punctuation_and_apply_immediately() {
        let mut value = memory();
        add_or_update_entry(
            &mut value,
            "Abin Varghese",
            &["Eben Vergis.".into()],
            MemoryKind::Person,
            "1",
            None,
        );
        let result = personalize(&mut value, "My name is Eben Vergis", "", "2");
        assert_eq!(result.text, "My name is Abin Varghese");
    }

    #[test]
    fn explicit_entries_can_handle_a_strong_phonetic_variant() {
        let mut value = memory();
        add_or_update_entry(
            &mut value,
            "Abin Varghese",
            &["Eben Vergis".into()],
            MemoryKind::Person,
            "1",
            None,
        );
        let result = personalize(&mut value, "My name is Eben Varges", "", "2");
        assert_eq!(result.text, "My name is Abin Varghese");
    }

    #[test]
    fn repeated_correction_applies_exact_alias() {
        let mut value = memory();
        learn_correction(
            &mut value,
            "Call Eben Vargis",
            "Call Abin Varghese",
            "call person",
            "1",
        );
        learn_correction(
            &mut value,
            "Call Eben Vargis",
            "Call Abin Varghese",
            "call person",
            "2",
        );
        let result = personalize(&mut value, "Call Eben Vargis", "call person", "3");
        assert_eq!(result.text, "Call Abin Varghese");
        assert_eq!(result.replacements.len(), 1);
    }

    #[test]
    fn learning_example_extracts_a_correction_and_tracks_acceptance() {
        let mut value = memory();
        let id = add_learning_example(
            &mut value,
            "I sent it to Eben Vargis",
            "I sent it to Eben Vargis",
            "learning-audio/example.wav",
            &[],
            "1",
        );
        let result = complete_learning_example(
            &mut value,
            &id,
            "I sent it to Abin Varghese",
            "send report",
            "2",
        );
        assert_eq!(result.status, "learned");
        assert_eq!(
            result.corrected.as_deref(),
            Some("I sent it to Abin Varghese")
        );
        assert_eq!(value.learning_examples[0].failures, 1);
        assert_eq!(value.corrections[0].original, "Eben Vargis");
        assert_eq!(value.corrections[0].corrected, "Abin Varghese");
        assert!(!value.entries[0].promoted);

        assert!(mark_learning_outcome(&mut value, &id, true, "3"));
        assert_eq!(value.learning_examples[0].status, "unchanged");
        assert_eq!(value.learning_examples[0].successes, 1);
        assert_eq!(value.recent_transcripts.len(), 0);
    }

    #[test]
    fn successful_future_applications_promote_the_linked_correction() {
        let mut value = memory();
        learn_correction(
            &mut value,
            "My name is Eben Arges",
            "My name is Abin Varghese",
            "name",
            "1",
        );
        let first = personalize(&mut value, "My name is Eben Arges", "name", "2");
        assert_eq!(first.text, "My name is Abin Varghese");
        assert_eq!(first.applied_entry_ids.len(), 1);

        let first_id = add_learning_example(
            &mut value,
            "My name is Eben Arges",
            &first.text,
            "learning-audio/one.wav",
            &first.applied_entry_ids,
            "2",
        );
        assert!(mark_learning_outcome(&mut value, &first_id, true, "3"));

        let second_id = add_learning_example(
            &mut value,
            "My name is Eben Arges",
            &first.text,
            "learning-audio/two.wav",
            &first.applied_entry_ids,
            "4",
        );
        assert!(mark_learning_outcome(&mut value, &second_id, true, "5"));
        assert_eq!(value.corrections[0].successes, 2);
        assert!(value.corrections[0].promoted);
        assert!(value.entries[0].promoted);
    }

    #[test]
    fn an_unobserved_editor_does_not_count_as_a_failed_correction() {
        let mut value = memory();
        let id = add_learning_example(
            &mut value,
            "hello",
            "hello",
            "learning-audio/example.wav",
            &[],
            "1",
        );
        assert!(mark_learning_observation_unavailable(&mut value, &id, "2"));
        assert_eq!(value.learning_examples[0].status, "unobserved");
        assert_eq!(value.learning_examples[0].failures, 0);
    }

    #[test]
    fn contradictory_candidates_require_a_margin() {
        let mut value = memory();
        learn_correction(&mut value, "Call Sam", "Call Sam Smith", "work", "1");
        learn_correction(&mut value, "Call Sam", "Call Sam Lee", "family", "2");
        let result = personalize(&mut value, "Call Sam", "unknown", "3");
        assert_eq!(result.text, "Call Sam");
    }

    #[test]
    fn context_can_disambiguate_two_names() {
        let mut value = memory();
        add_or_update_entry(
            &mut value,
            "Sam Smith",
            &["Sam".into()],
            MemoryKind::Person,
            "1",
            None,
        );
        add_or_update_entry(
            &mut value,
            "Sam Lee",
            &["Sam".into()],
            MemoryKind::Person,
            "1",
            None,
        );
        value.entries[0].contexts.push("work report".into());
        value.entries[0].confidence = 0.95;
        value.entries[1].contexts.push("family dinner".into());
        value.entries[1].confidence = 0.95;
        let result = personalize(&mut value, "Send this to Sam", "work report", "2");
        assert_eq!(result.text, "Send this to Sam Smith");
    }

    #[test]
    fn deleting_a_correction_removes_its_automatic_alias() {
        let mut value = memory();
        let learned = learn_correction(
            &mut value,
            "Call Eben Vargis",
            "Call Abin Varghese",
            "call",
            "1",
        );
        learn_correction(
            &mut value,
            "Call Eben Vargis",
            "Call Abin Varghese",
            "call",
            "2",
        );
        assert!(delete_correction(&mut value, &learned.learned[0].id));
        assert!(value.entries[0].aliases.is_empty());
    }

    #[test]
    fn personal_names_and_compound_terms_inject_reliably() {
        let mut value = VoiceMemory {
            schema_version: CURRENT_SCHEMA_VERSION,
            entries: default_baseline_entries(),
            corrections: Vec::new(),
            recent_transcripts: Vec::new(),
            learning_examples: Vec::new(),
        };

        // Test multi-word spoken compound term matching single canonical word
        let r1 = personalize(&mut value, "open anti gravity right now", "ide", "1");
        assert_eq!(r1.text, "open Antigravity right now");

        // Test AI tool compound term
        let r2 = personalize(&mut value, "ask chat gpt for help", "ai", "2");
        assert_eq!(r2.text, "ask ChatGPT for help");

        // Test user handle
        let r3 = personalize(&mut value, "I am epin ogees", "profile", "3");
        assert_eq!(r3.text, "I am epinogees");

        // Test personal name phonetic alias
        let r4 = personalize(&mut value, "Created by abin vargis on github", "credits", "4");
        assert_eq!(r4.text, "Created by Abin Varghese on github");
    }
}
