// Small helpers used across the app: date, text normalization, paths, CSV load.

use chrono::{Local, NaiveDate};
use std::path::{Path, PathBuf};
use unicode_normalization::UnicodeNormalization;

use crate::model::Word;

/// Current local date (no time component).
pub fn today() -> NaiveDate {
    Local::now().date_naive()
}

/// Normalize an answer string for lenient comparison in dictation mode.
/// Trims, lowercases, applies Unicode NFC, strips common punctuation and
/// collapses inner whitespace to single spaces.
pub fn norm(s: &str) -> String {
    let punctuation = ['.', ',', '!', '?', ';', ':', '"', '\'', '„', '“'];
    s.trim()
        .to_lowercase()
        .nfc()
        .filter(|c| !punctuation.contains(c))
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Directory next to the running executable, falling back to the current
/// working directory, then to ".".
pub fn app_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(Path::to_path_buf))
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."))
}

/// Load enabled vocabulary rows from vocabulary.csv, searching a few
/// candidate locations relative to `base`.
pub fn load_words(base: &Path) -> Result<Vec<Word>, String> {
    let candidates = [
        base.join("vocabulary.csv"),
        base.join("data").join("vocabulary.csv"),
        PathBuf::from("data/vocabulary.csv"),
    ];
    let path = candidates
        .iter()
        .find(|p| p.exists())
        .ok_or("找不到 vocabulary.csv。请把它放在程序旁边或 data 文件夹中。")?;

    let mut rdr = csv::ReaderBuilder::new()
        .flexible(true)
        .from_path(path)
        .map_err(|e| e.to_string())?;

    let mut words = Vec::new();
    for row in rdr.deserialize() {
        let w: Word = row.map_err(|e| format!("CSV 格式错误: {e}"))?;
        if w.enabled {
            words.push(w);
        }
    }

    if words.is_empty() {
        return Err("CSV 中没有启用的词条。".into());
    }
    Ok(words)
}
