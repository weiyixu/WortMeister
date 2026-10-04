// Small, filesystem-agnostic helpers shared across platforms: date handling,
// answer normalization and CSV parsing from an in-memory string.
//
// Note: unlike the desktop util.rs, this core module does NOT touch the
// filesystem. iOS runs in a sandbox, so the host app (Swift) is responsible
// for reading/writing files and simply passes CSV/JSON text to the core.

use chrono::{Local, NaiveDate};
use unicode_normalization::UnicodeNormalization;

use crate::model::Word;

/// Current local date (no time component).
pub fn today() -> NaiveDate {
    Local::now().date_naive()
}

/// Normalize an answer string for lenient comparison in dictation mode.
///
/// Trims, lowercases, applies Unicode NFC, strips common punctuation, folds
/// German umlauts and eszett to their ASCII transcriptions (so `ue == ue`,
/// `ae == ae`, `oe == oe`, `ss == ss`) and collapses inner whitespace to
/// single spaces. Folding both the expected answer and the user input the
/// same way makes typing `Fuesse` accepted for `Fuesse`.
pub fn norm(s: &str) -> String {
    let punctuation = ['.', ',', '!', '?', ';', ':', '"', '\'', '\u{201e}', '\u{201c}'];
    let lowered: String = s
        .trim()
        .to_lowercase()
        .nfc()
        .filter(|c| !punctuation.contains(c))
        .collect();

    // Fold umlauts/eszett to ASCII so both spellings compare equal.
    let mut folded = String::with_capacity(lowered.len());
    for c in lowered.chars() {
        match c {
            '\u{e4}' => folded.push_str("ae"), // a-umlaut
            '\u{f6}' => folded.push_str("oe"), // o-umlaut
            '\u{fc}' => folded.push_str("ue"), // u-umlaut
            '\u{df}' => folded.push_str("ss"), // eszett
            other => folded.push(other),
        }
    }

    folded.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Parse enabled vocabulary rows from an in-memory CSV string.
/// The host app loads the CSV bytes (bundled resource on iOS, file beside the
/// executable on desktop) and passes the text here.
pub fn parse_words(csv_text: &str) -> Result<Vec<Word>, String> {
    let mut rdr = csv::ReaderBuilder::new()
        .flexible(true)
        .from_reader(csv_text.as_bytes());

    let mut words = Vec::new();
    for row in rdr.deserialize() {
        let w: Word = row.map_err(|e| format!("CSV format error: {e}"))?;
        if w.enabled {
            words.push(w);
        }
    }

    if words.is_empty() {
        return Err("No enabled entries in CSV.".into());
    }
    Ok(words)
}
