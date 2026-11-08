//! Turns a raw speech transcript into clean text ready to type.
//!
//! The speech model already adds punctuation and capitals, so formatting is deliberately
//! conservative: it repairs the specific slips that were observed (missing spaces around
//! currency and punctuation, a lone lowercase "I", stuttered function words, filler
//! sounds) and leaves everything else alone.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct FormatOptions {
    /// Drop "um", "uh" and similar.
    pub remove_fillers: bool,
    /// Turn "comma", "new line", "question mark" into punctuation. Off by default because it
    /// rewrites ordinary uses of those words.
    pub spoken_punctuation: bool,
    /// Start each sentence with a capital.
    pub auto_capitalize: bool,
    /// Repair spacing, the pronoun "I" and stuttered words.
    pub smart_format: bool,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            remove_fillers: true,
            spoken_punctuation: false,
            auto_capitalize: true,
            smart_format: true,
        }
    }
}

fn regex(pattern: &str) -> Regex {
    Regex::new(&format!("(?i){pattern}")).expect("static pattern is valid")
}

static FILLERS: LazyLock<Regex> =
    LazyLock::new(|| regex(r"\b(?:um+|uh+|uhm+|erm+|ah+|hmm+)\b[,.]?\s*"));

static SPOKEN: LazyLock<Vec<(Regex, &'static str)>> = LazyLock::new(|| {
    vec![
        (regex(r"\s*\bnew paragraph\b[,.]?\s*"), "\n\n"),
        (regex(r"\s*\bnew line\b[,.]?\s*"), "\n"),
        (regex(r"\s*\bcomma\b[,.]?"), ","),
        (regex(r"\s*\b(?:period|full stop)\b[,.]?"), "."),
        (regex(r"\s*\bquestion mark\b[,.]?"), "?"),
        (regex(r"\s*\bexclamation (?:mark|point)\b[,.]?"), "!"),
        (regex(r"\s*\bsemicolon\b[,.]?"), ";"),
        (regex(r"\s*\bcolon\b[,.]?"), ":"),
    ]
});

static SPACE_BEFORE_PUNCT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[ \t]+([,.;:!?])").unwrap());
static MULTI_SPACE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[ \t]{2,}").unwrap());
static SPACE_BEFORE_NEWLINE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[ \t]+\n").unwrap());
static CURRENCY: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"([A-Za-z])([$€£¥])(\d)").unwrap());
static COMMA_SPACING: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"([,;])([A-Za-z])").unwrap());
static SENTENCE_GAP: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"([a-z])([.?!])([A-Z])").unwrap());
static SENTENCE_START: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(^|[.!?]\s+|\n)([a-z])").unwrap());
static TOKEN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[A-Za-z]+(?:['’][A-Za-z]+)*|[^A-Za-z]+").unwrap());

/// Short function words that people stutter ("the the"). Words that are legitimately
/// doubled ("had had", "that that") are left out on purpose.
const STUTTER_WORDS: &[&str] = &[
    "the", "a", "an", "to", "of", "in", "is", "and", "i", "we", "you", "it", "so", "but", "on",
    "for", "at", "my", "your", "our", "this",
];

fn is_word(token: &str) -> bool {
    token
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic())
}

/// True for the "i" of "i.e." so it is not turned into "I".
fn is_dotted_abbreviation(tokens: &[&str], index: usize) -> bool {
    tokens.get(index + 1) == Some(&".")
        && tokens
            .get(index + 2)
            .is_some_and(|t| is_word(t) && t.len() == 1)
}

/// Fix the pronoun "I" and drop stuttered words in one pass over words and separators.
fn fix_words(text: &str) -> String {
    let tokens: Vec<&str> = TOKEN.find_iter(text).map(|m| m.as_str()).collect();
    let mut out = String::with_capacity(text.len());
    let mut previous_word: Option<String> = None;

    for (index, token) in tokens.iter().enumerate() {
        if !is_word(token) {
            if !token.chars().all(|c| c == ' ' || c == '\t') {
                previous_word = None;
            }
            out.push_str(token);
            continue;
        }
        let lower = token.to_lowercase();
        if previous_word.as_deref() == Some(lower.as_str())
            && STUTTER_WORDS.contains(&lower.as_str())
        {
            while out.ends_with(' ') || out.ends_with('\t') {
                out.pop();
            }
            continue;
        }
        let is_pronoun = lower == "i" || lower.starts_with("i'") || lower.starts_with("i’");
        if is_pronoun && !is_dotted_abbreviation(&tokens, index) {
            out.push('I');
            out.push_str(&token[1..]);
        } else {
            out.push_str(token);
        }
        previous_word = Some(lower);
    }
    out
}

/// A full stop that ends an abbreviation ("i.e.", "e.g.", "etc.", "Dr.") does not end a sentence.
static ABBREVIATION_END: LazyLock<Regex> =
    LazyLock::new(|| regex(r"(?:\b[a-z]\.[a-z]|\b(?:etc|vs|mr|mrs|ms|dr|approx))\.$"));

fn capitalize_sentences(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut copied_to = 0;
    for caps in SENTENCE_START.captures_iter(text) {
        let Some(whole) = caps.get(0) else { continue };
        let lead = &caps[1];
        if lead.starts_with('.') && ABBREVIATION_END.is_match(&text[..whole.start() + 1]) {
            continue;
        }
        out.push_str(&text[copied_to..whole.start()]);
        out.push_str(lead);
        out.push_str(&caps[2].to_uppercase());
        copied_to = whole.end();
    }
    out.push_str(&text[copied_to..]);
    out
}

/// Tidy a raw speech transcript into text ready to insert.
pub fn format_transcript(input: &str, options: FormatOptions) -> String {
    let mut text = input.trim().to_string();

    if options.remove_fillers {
        text = FILLERS.replace_all(&text, "").into_owned();
    }
    if options.spoken_punctuation {
        for (pattern, replacement) in SPOKEN.iter() {
            text = pattern.replace_all(&text, *replacement).into_owned();
        }
    }

    text = SPACE_BEFORE_PUNCT.replace_all(&text, "${1}").into_owned();
    text = MULTI_SPACE.replace_all(&text, " ").into_owned();
    text = SPACE_BEFORE_NEWLINE.replace_all(&text, "\n").into_owned();

    if options.smart_format {
        text = CURRENCY.replace_all(&text, "${1} ${2}${3}").into_owned();
        text = COMMA_SPACING.replace_all(&text, "${1} ${2}").into_owned();
        text = SENTENCE_GAP
            .replace_all(&text, "${1}${2} ${3}")
            .into_owned();
        text = fix_words(&text);
    }

    let text = text.trim().to_string();
    if options.auto_capitalize {
        capitalize_sentences(&text)
    } else {
        text
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn defaults() -> FormatOptions {
        FormatOptions::default()
    }

    fn spoken_on() -> FormatOptions {
        FormatOptions {
            spoken_punctuation: true,
            ..defaults()
        }
    }

    #[test]
    fn removes_fillers_and_capitalizes_the_first_letter() {
        assert_eq!(
            format_transcript("um so this is uh a test", defaults()),
            "So this is a test"
        );
        assert_eq!(
            format_transcript("Um, so the report is done.", defaults()),
            "So the report is done."
        );
    }

    #[test]
    fn converts_spoken_punctuation_when_enabled() {
        assert_eq!(
            format_transcript("hello comma world period", spoken_on()),
            "Hello, world."
        );
    }

    #[test]
    fn handles_new_line_and_new_paragraph() {
        assert_eq!(
            format_transcript(
                "first line new line second line new paragraph third",
                spoken_on()
            ),
            "First line\nSecond line\n\nThird"
        );
    }

    #[test]
    fn leaves_ordinary_words_alone_by_default() {
        assert_eq!(
            format_transcript("the period of time", defaults()),
            "The period of time"
        );
    }

    #[test]
    fn capitalizes_after_sentence_ending_punctuation() {
        assert_eq!(
            format_transcript("it works. really? yes", defaults()),
            "It works. Really? Yes"
        );
    }

    #[test]
    fn leaves_text_untouched_when_every_option_is_off() {
        let off = FormatOptions {
            remove_fillers: false,
            spoken_punctuation: false,
            auto_capitalize: false,
            smart_format: false,
        };
        assert_eq!(
            format_transcript("um hello comma there", off),
            "um hello comma there"
        );
        assert_eq!(format_transcript("is$245", off), "is$245");
    }

    #[test]
    fn returns_an_empty_string_for_blank_input() {
        assert_eq!(format_transcript("   ", defaults()), "");
    }

    #[test]
    fn adds_the_missing_space_before_a_currency_symbol() {
        assert_eq!(
            format_transcript(
                "The invoice total is$245 and it is due on March 3.",
                defaults()
            ),
            "The invoice total is $245 and it is due on March 3."
        );
        assert_eq!(
            format_transcript("It costs $5 and €7.", defaults()),
            "It costs $5 and €7."
        );
    }

    #[test]
    fn fixes_the_pronoun_i() {
        assert_eq!(
            format_transcript("i think i can do it. i'm sure i'll try", defaults()),
            "I think I can do it. I'm sure I'll try"
        );
    }

    #[test]
    fn does_not_capitalize_the_i_of_ie() {
        assert_eq!(
            format_transcript("use fruit, i.e. apples", defaults()),
            "Use fruit, i.e. apples"
        );
    }

    #[test]
    fn does_not_capitalize_after_an_abbreviation() {
        assert_eq!(
            format_transcript("ask Dr. smith about it", defaults()),
            "Ask Dr. smith about it"
        );
        assert_eq!(
            format_transcript("bring pens, paper etc. and a bag", defaults()),
            "Bring pens, paper etc. and a bag"
        );
        assert_eq!(
            format_transcript("it works. then it stops", defaults()),
            "It works. Then it stops"
        );
    }

    #[test]
    fn drops_stuttered_function_words_but_not_real_doubles() {
        assert_eq!(
            format_transcript("the the quick brown fox", defaults()),
            "The quick brown fox"
        );
        assert_eq!(
            format_transcript("we we should go to to the shop", defaults()),
            "We should go to the shop"
        );
        assert_eq!(
            format_transcript("she had had a plan", defaults()),
            "She had had a plan"
        );
        assert_eq!(
            format_transcript("I said that that works", defaults()),
            "I said that that works"
        );
    }

    #[test]
    fn a_repeat_across_a_comma_or_sentence_is_kept() {
        assert_eq!(format_transcript("so, so true", defaults()), "So, so true");
        assert_eq!(
            format_transcript("It is. It is.", defaults()),
            "It is. It is."
        );
    }

    #[test]
    fn repairs_spacing_after_commas_and_sentences() {
        assert_eq!(
            format_transcript("red,green and blue", defaults()),
            "Red, green and blue"
        );
        assert_eq!(
            format_transcript("it is done.Next step", defaults()),
            "It is done. Next step"
        );
    }

    #[test]
    fn leaves_numbers_urls_and_emails_alone() {
        assert_eq!(
            format_transcript("1,000 apples at example.com", defaults()),
            "1,000 apples at example.com"
        );
        assert_eq!(
            format_transcript("mail john@example.com now", defaults()),
            "Mail john@example.com now"
        );
        assert_eq!(format_transcript("pi is 3.14", defaults()), "Pi is 3.14");
    }

    #[test]
    fn removes_space_before_punctuation() {
        assert_eq!(
            format_transcript("hello , world !", defaults()),
            "Hello, world!"
        );
    }

    #[test]
    fn is_idempotent() {
        let once = format_transcript("um i think the the budget is$245,ok.fine", defaults());
        assert_eq!(format_transcript(&once, defaults()), once);
    }
}
