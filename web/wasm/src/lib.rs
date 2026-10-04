// WebAssembly bindings for the WortMeister core.
//
// This exposes the shared Rust logic (CSV parsing, SM-2 scheduling, dictation
// checking) to JavaScript running in the browser PWA. We call the core modules
// directly (not the C FFI) and exchange data as JSON strings, which map
// cleanly to/from JS objects via JSON.parse / JSON.stringify.

use wasm_bindgen::prelude::*;

use wortmeister_core::model::{ProgressFile, Word};

use wortmeister_core::session;
use wortmeister_core::srs::Grade;
use wortmeister_core::util::{norm, parse_words};

use serde_json::json;

/// Core version string.
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Parse vocabulary CSV text. Returns a JSON array of Word objects, or throws
/// a JS error with the message on failure.
#[wasm_bindgen]
pub fn parse_words_json(csv: &str) -> Result<String, JsError> {
    let words = parse_words(csv).map_err(|e| JsError::new(&e))?;
    serde_json::to_string(&words).map_err(|e| JsError::new(&e.to_string()))
}

/// Distinct levels (prefixed with "all") for a JSON array of words.
#[wasm_bindgen]
pub fn levels_json(words_json: &str) -> Result<String, JsError> {
    let words: Vec<Word> = serde_json::from_str(words_json)
        .map_err(|e| JsError::new(&format!("bad words: {e}")))?;

    serde_json::to_string(&session::levels(&words))
        .map_err(|e| JsError::new(&e.to_string()))
}

/// Build today's review queue. Inputs are JSON strings; returns a JSON array of
/// word IDs. `progress_json` may be "" or "null" for a fresh start.
#[wasm_bindgen]
pub fn make_queue_json(
    words_json: &str,
    progress_json: &str,
    level: &str,
    lesson: &str,
    daily_limit: usize,
) -> Result<String, JsError> {
    let words: Vec<Word> = serde_json::from_str(words_json)
        .map_err(|e| JsError::new(&format!("bad words: {e}")))?;
    let progress = parse_progress(progress_json)?;
    let queue = session::make_queue(&words, &progress, level, lesson, daily_limit);

    serde_json::to_string(&queue).map_err(|e| JsError::new(&e.to_string()))
}

/// Apply a grade to a card. `grade` is "again"|"hard"|"good"|"easy". Returns
/// the updated ProgressFile as JSON for the caller to persist (localStorage).
#[wasm_bindgen]
pub fn grade_card_json(
    progress_json: &str,
    word_id: &str,
    grade: &str,
) -> Result<String, JsError> {
    let mut progress = parse_progress(progress_json)?;
    let g = parse_grade(grade)?;
    session::grade_card(&mut progress, word_id, g);
    serde_json::to_string(&progress).map_err(|e| JsError::new(&e.to_string()))
}

/// Lenient dictation check: true when the normalized forms match.
#[wasm_bindgen]
pub fn check_answer(expected: &str, actual: &str) -> bool {
    norm(expected) == norm(actual)
}

/// Count how many of `words` are currently due given saved progress. Handy for
/// showing a badge on the home screen. Returns JSON `{ "due": n, "total": m }`.
#[wasm_bindgen]
pub fn due_summary_json(
    words_json: &str,
    progress_json: &str,
    level: &str,
    lesson: &str,
) -> Result<String, JsError> {
    let words: Vec<Word> = serde_json::from_str(words_json)
        .map_err(|e| JsError::new(&format!("bad words: {e}")))?;
    let progress = parse_progress(progress_json)?;
    // A large limit so we count all due cards, not a capped queue.

    let due = session::make_queue(&words, &progress, level, lesson, usize::MAX).len();
    let total: usize = serde_json::from_str::<Vec<serde_json::Value>>(words_json)
        .map(|v| v.len())
        .unwrap_or(0);
    Ok(json!({ "due": due, "total": total }).to_string())
}

/// Aggregate learning statistics as JSON (studied/new/learned/accuracy/streak
/// and more). Input: words JSON + progress JSON.
#[wasm_bindgen]
pub fn stats_json(words_json: &str, progress_json: &str) -> Result<String, JsError> {
    let words: Vec<Word> = serde_json::from_str(words_json)
        .map_err(|e| JsError::new(&format!("bad words: {e}")))?;
    let progress = parse_progress(progress_json)?;
    serde_json::to_string(&session::stats(&words, &progress))
        .map_err(|e| JsError::new(&e.to_string()))
}


// --- helpers ---------------------------------------------------------------

/// Parse a ProgressFile from JSON, treating empty/null as a fresh file.
fn parse_progress(s: &str) -> Result<ProgressFile, JsError> {
    let trimmed = s.trim();
    if trimmed.is_empty() || trimmed == "null" {
        return Ok(ProgressFile::default());
    }
    serde_json::from_str(trimmed).map_err(|e| JsError::new(&format!("bad progress: {e}")))
}

/// Map a grade string to the core Grade enum.
fn parse_grade(s: &str) -> Result<Grade, JsError> {
    match s {
        "again" => Ok(Grade::Again),
        "hard" => Ok(Grade::Hard),
        "good" => Ok(Grade::Good),
        "easy" => Ok(Grade::Easy),
        other => Err(JsError::new(&format!("unknown grade: {other}"))),
    }
}
