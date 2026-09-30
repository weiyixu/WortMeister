// Small helpers used across the app: date, text normalization, paths, CSV load.

use chrono::{Local, NaiveDate};
use std::fs;
use std::path::{Path, PathBuf};
use unicode_normalization::UnicodeNormalization;

use crate::model::{ProgressFile, Word};

/// Current local date (no time component).
pub fn today() -> NaiveDate {
    Local::now().date_naive()
}

/// Normalize an answer string for lenient comparison in dictation mode.
///
/// Trims, lowercases, applies Unicode NFC, strips common punctuation, folds
/// German umlauts and eszett to their ASCII transcriptions (so `ue == ü`,
/// `ae == ä`, `oe == ö`, `ss == ß`) and collapses inner whitespace to single
/// spaces. Folding both the expected answer and the user input the same way
/// makes typing `Fuesse` accepted for `Füße`.
pub fn norm(s: &str) -> String {
    let punctuation = ['.', ',', '!', '?', ';', ':', '"', '\'', '„', '“'];
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
            'ä' => folded.push_str("ae"),
            'ö' => folded.push_str("oe"),
            'ü' => folded.push_str("ue"),
            'ß' => folded.push_str("ss"),
            other => folded.push(other),
        }
    }

    folded.split_whitespace().collect::<Vec<_>>().join(" ")
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

/// Load the progress file. Returns the parsed progress plus an optional
/// user-facing notice.
///
/// If the file is missing we start fresh with no notice. If the file exists
/// but cannot be parsed (corrupted or from an incompatible version) we do NOT
/// silently discard it: the damaged file is renamed to a timestamped backup
/// so the user can recover it, and a notice is returned explaining what
/// happened.
pub fn load_progress(path: &Path) -> (ProgressFile, String) {
    let raw = match fs::read_to_string(path) {
        Ok(s) => s,
        // No file yet (first run) or unreadable: start fresh, no notice.
        Err(_) => return (ProgressFile::default(), String::new()),
    };

    match serde_json::from_str::<ProgressFile>(&raw) {
        Ok(progress) => (progress, String::new()),
        Err(e) => {
            let notice = match backup_corrupt_file(path) {
                Ok(backup) => format!(
                    "进度文件已损坏，无法解析（{e}）。已备份为 {} 并新建空进度。",
                    backup.display()
                ),
                Err(be) => format!(
                    "进度文件已损坏，无法解析（{e}），且备份失败（{be}）。已使用空进度，请手动备份 progress.json。"
                ),
            };
            (ProgressFile::default(), notice)
        }
    }
}

/// Rename a corrupt progress file to a timestamped ".bak" sibling so it is
/// preserved rather than overwritten. Returns the backup path on success.
fn backup_corrupt_file(path: &Path) -> std::io::Result<PathBuf> {
    let stamp = Local::now().format("%Y%m%d-%H%M%S");
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("progress.json");
    let backup = path.with_file_name(format!("{file_name}.corrupt-{stamp}.bak"));
    fs::rename(path, &backup)?;
    Ok(backup)
}


