//! Best-effort focused-editor observation for automatic post-paste learning.
//!
//! Windows UI Automation exposes the focused text control through ValuePattern or TextPattern.
//! Applications that do not expose either pattern simply return `None`; the manual teaching flow
//! remains available for those apps.

#[derive(Clone, Debug)]
pub struct FocusedText {
    pub identity: String,
    pub text: String,
}

/// UI Automation may normalize line breaks and repeated spaces differently from the text Murmur
/// pasted. Use the same whitespace-normalized representation when waiting for the insertion to
/// become visible, while keeping the original snapshots for precise diff extraction.
pub fn contains_inserted_text(document: &str, inserted: &str) -> bool {
    let normalize = |value: &str| value.split_whitespace().collect::<Vec<_>>().join(" ");
    let document = normalize(document);
    let inserted = normalize(inserted);
    !inserted.is_empty() && document.contains(&inserted)
}

/// Extract the corrected version of the inserted phrase from two snapshots of the focused
/// editor. The snapshots may contain the entire document; only a replacement that overlaps the
/// most recent inserted phrase is returned. This prevents unrelated edits elsewhere in a document
/// from being promoted into voice memory.
pub fn extract_edited_phrase(before: &str, after: &str, inserted: &str) -> Option<String> {
    if before == after || inserted.is_empty() {
        return None;
    }

    // The pasted phrase can occur more than once in a document. The newest occurrence is the one
    // most likely to be the dictation just inserted.
    let inserted_byte_start = before.rfind(inserted)?;
    let inserted_start = before[..inserted_byte_start].chars().count();
    let inserted_end = inserted_start + inserted.chars().count();
    let before_chars: Vec<char> = before.chars().collect();
    let after_chars: Vec<char> = after.chars().collect();

    let mut prefix = 0usize;
    while prefix < before_chars.len()
        && prefix < after_chars.len()
        && before_chars[prefix] == after_chars[prefix]
    {
        prefix += 1;
    }

    let mut suffix = 0usize;
    while suffix < before_chars.len().saturating_sub(prefix)
        && suffix < after_chars.len().saturating_sub(prefix)
        && before_chars[before_chars.len() - 1 - suffix]
            == after_chars[after_chars.len() - 1 - suffix]
    {
        suffix += 1;
    }

    let before_end = before_chars.len().saturating_sub(suffix);
    let after_end = after_chars.len().saturating_sub(suffix);
    let overlaps_phrase = (prefix < inserted_end && before_end > inserted_start)
        || (prefix == before_end && prefix >= inserted_start && prefix <= inserted_end);
    if !overlaps_phrase {
        return None;
    }

    // Clamp the old changed range to the inserted phrase, then rebuild the phrase with the new
    // text. For an insertion at the phrase boundary, the old range is empty and the new text is
    // still included.
    let local_start = prefix.max(inserted_start).min(inserted_end);
    let local_end = before_end.max(inserted_start).min(inserted_end);
    let replacement: String = after_chars[prefix.min(after_chars.len())..after_end]
        .iter()
        .collect();
    let mut corrected: String = before_chars[inserted_start..local_start].iter().collect();
    corrected.push_str(&replacement);
    corrected.extend(before_chars[local_end..inserted_end].iter());
    let corrected = corrected.trim().to_string();
    (!corrected.is_empty() && corrected != inserted).then_some(corrected)
}

#[cfg(windows)]
pub fn read_focused_text() -> Option<FocusedText> {
    use uiautomation::patterns::{UITextPattern, UIValuePattern};
    use uiautomation::UIAutomation;

    let automation = UIAutomation::new().ok()?;
    let focused = automation.get_focused_element().ok()?;
    let process = focused.get_process_id().ok()?;
    let runtime_id = focused.get_runtime_id().ok()?;
    let identity = format!("{process}:{runtime_id:?}");
    let text = if let Ok(value) = focused.get_pattern::<UIValuePattern>() {
        value.get_value().ok()?
    } else {
        let text = focused.get_pattern::<UITextPattern>().ok()?;
        text.get_document_range().ok()?.get_text(-1).ok()?
    };
    Some(FocusedText { identity, text })
}

#[cfg(not(windows))]
pub fn read_focused_text() -> Option<FocusedText> {
    None
}

#[cfg(test)]
mod tests {
    use super::{contains_inserted_text, extract_edited_phrase};

    #[test]
    fn finds_an_inserted_phrase_across_normalized_whitespace() {
        assert!(contains_inserted_text(
            "Before\n  dictated   quasar after",
            "dictated quasar"
        ));
    }

    #[test]
    fn does_not_match_empty_insertions() {
        assert!(!contains_inserted_text("anything", "  \n  "));
    }

    #[test]
    fn extracts_a_single_word_replacement_from_a_document() {
        assert_eq!(
            extract_edited_phrase(
                "Before dictated quasar after",
                "Before dictated kazar after",
                "dictated quasar"
            ),
            Some("dictated kazar".into())
        );
    }

    #[test]
    fn extracts_an_inserted_suffix_without_learning_surrounding_text() {
        assert_eq!(
            extract_edited_phrase("Say hello there", "Say hello there!", "hello there"),
            Some("hello there!".into())
        );
    }

    #[test]
    fn ignores_an_unrelated_document_edit() {
        assert_eq!(
            extract_edited_phrase(
                "Fix this dictated phrase",
                "Fix that dictated phrase",
                "dictated phrase"
            ),
            None
        );
    }
}
