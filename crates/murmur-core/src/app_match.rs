//! Picks the installed app a person meant by what they said. "chrome" finds "Google Chrome",
//! "vlc" finds "VLC media player". Every spoken word must start a word in the app's name.
//! Command shells and uninstallers are never matched this way, so a fragment like "power"
//! can never launch PowerShell by accident.

use regex::Regex;
use std::sync::LazyLock;

static BLOCKED: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"powershell|command prompt|terminal|\bcmd\b|\bwsl\b|\bbash\b|regedit|pwsh|script|uninstall").unwrap()
});

/// True when `token` starts a word in `haystack` (both already lowercase).
fn starts_a_word(haystack: &str, token: &str) -> bool {
    haystack.match_indices(token).any(|(position, _)| {
        position == 0
            || !haystack[..position]
                .chars()
                .next_back()
                .is_some_and(char::is_alphanumeric)
    })
}

/// Calculate Levenshtein edit distance for typo tolerance.
fn edit_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let m = a_chars.len();
    let n = b_chars.len();
    if m == 0 {
        return n;
    }
    if n == 0 {
        return m;
    }
    let mut dp = vec![vec![0; n + 1]; m + 1];
    for i in 0..=m {
        dp[i][0] = i;
    }
    for j in 0..=n {
        dp[0][j] = j;
    }
    for i in 1..=m {
        for j in 1..=n {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            dp[i][j] = (dp[i - 1][j] + 1)
                .min(dp[i][j - 1] + 1)
                .min(dp[i - 1][j - 1] + cost);
        }
    }
    dp[m][n]
}

/// Index of the best-matching name, or `None`. Exact names win, then names that start with
/// what was said, then the shortest name that contains every spoken word, with compact
/// whitespace-agnostic and typo-tolerant fallback.
pub fn best_match(spoken: &str, names: &[String]) -> Option<usize> {
    let spoken = spoken.trim().to_lowercase();
    let tokens: Vec<&str> = spoken.split_whitespace().collect();
    if tokens.is_empty() {
        return None;
    }
    let spoken_compact: String = spoken.chars().filter(|c| c.is_alphanumeric()).collect();

    names
        .iter()
        .enumerate()
        .filter_map(|(index, name)| {
            let lower = name.to_lowercase();
            if BLOCKED.is_match(&lower) {
                return None;
            }
            let name_compact: String = lower.chars().filter(|c| c.is_alphanumeric()).collect();

            // Rank 0: Exact match (standard or compact, e.g. "chat gpt" == "chatgpt")
            if lower == spoken || (!spoken_compact.is_empty() && name_compact == spoken_compact) {
                return Some((0, lower.len(), index));
            }

            // Rank 1: Prefix match (e.g. "chat gpt" starts "chatgpt...")
            if lower.starts_with(&spoken)
                || (!spoken_compact.is_empty() && name_compact.starts_with(&spoken_compact))
            {
                return Some((1, lower.len(), index));
            }

            // Rank 2: All spoken tokens start a word in the app name
            if tokens.iter().all(|t| starts_a_word(&lower, t)) {
                return Some((2, lower.len(), index));
            }

            // Rank 3: Typo-tolerant fuzzy match (e.g. "chat gbt" or "chagpt" matching "chatgpt")
            if spoken_compact.len() >= 4 && name_compact.len() >= 4 {
                let dist = edit_distance(&spoken_compact, &name_compact);
                if dist <= 2 {
                    return Some((3 + dist, lower.len(), index));
                }
            }

            None
        })
        .min()
        .map(|(_, _, index)| index)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn installed() -> Vec<String> {
        [
            "Discord",
            "Excel",
            "Google Chrome",
            "Install movie codec",
            "Microsoft Teams",
            "Microsoft Edge",
            "Outlook",
            "Visual Studio Code",
            "VLC media player",
            "VLC media player - reset preferences and cache files",
            "Windows PowerShell",
            "Word",
            "Uninstall Foo",
            "Command Prompt",
            "Windows Terminal",
            "Knowledge Base",
        ]
        .iter()
        .map(ToString::to_string)
        .collect()
    }

    fn found(spoken: &str) -> Option<String> {
        let names = installed();
        best_match(spoken, &names).map(|i| names[i].clone())
    }

    #[test]
    fn finds_apps_whose_names_start_with_a_different_word() {
        assert_eq!(found("chrome").as_deref(), Some("Google Chrome"));
        assert_eq!(
            found("visual studio code").as_deref(),
            Some("Visual Studio Code")
        );
        assert_eq!(found("teams").as_deref(), Some("Microsoft Teams"));
        assert_eq!(found("edge").as_deref(), Some("Microsoft Edge"));
    }

    #[test]
    fn prefers_exact_then_prefix_then_shortest() {
        assert_eq!(found("word").as_deref(), Some("Word"));
        assert_eq!(found("vlc").as_deref(), Some("VLC media player"));
        assert_eq!(found("outlook").as_deref(), Some("Outlook"));
    }

    #[test]
    fn never_matches_shells_or_uninstallers() {
        assert_eq!(found("power"), None);
        assert_eq!(found("powershell"), None);
        assert_eq!(found("terminal"), None);
        assert_eq!(found("command prompt"), None);
        assert_eq!(found("uninstall foo"), None);
    }

    #[test]
    fn does_not_match_in_the_middle_of_a_word() {
        assert_eq!(found("ledge"), None);
        assert_eq!(found("odec"), None);
    }

    #[test]
    fn unknown_or_empty_input_matches_nothing() {
        assert_eq!(found("spotify"), None);
        assert_eq!(found(""), None);
        assert_eq!(found("   "), None);
    }

    #[test]
    fn matches_chatgpt_with_spaces_and_minor_typo() {
        let names = vec![
            "ChatGPT".to_string(),
            "Figma".to_string(),
            "Antigravity".to_string(),
        ];
        assert_eq!(best_match("chat gpt", &names), Some(0));
        assert_eq!(best_match("chatgpt", &names), Some(0));
        assert_eq!(best_match("chat gbt", &names), Some(0));
        assert_eq!(best_match("anti gravity", &names), Some(2));
        assert_eq!(best_match("fig ma", &names), Some(1));
    }
}
