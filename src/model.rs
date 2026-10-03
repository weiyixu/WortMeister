// Data model types: vocabulary words and learning progress.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::util::today;

/// A single vocabulary entry loaded from the CSV store.
#[derive(Clone, Debug, Deserialize)]
pub struct Word {
    pub id: String,
    pub level: String,
    pub lesson: String,
    pub german: String,
    pub chinese: String,
    pub example: String,
    pub example_zh: String,
    pub tags: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

/// Spaced-repetition state for one card, keyed by Word::id.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CardProgress {
    pub repetitions: u32,
    pub interval_days: i64,
    pub ease: f32,
    pub due: NaiveDate,
    pub correct: u32,
    pub wrong: u32,
    pub last_review: Option<NaiveDate>,
}

impl CardProgress {
    pub fn new() -> Self {
        Self {
            repetitions: 0,
            interval_days: 0,
            ease: 2.5,
            due: today(),
            correct: 0,
            wrong: 0,
            last_review: None,
        }
    }
}

/// The persisted progress file (progress.json).
#[derive(Default, Serialize, Deserialize)]
pub struct ProgressFile {
    pub cards: HashMap<String, CardProgress>,
    pub daily_counts: HashMap<String, u32>,
}

/// User-configurable application settings, persisted to settings.ini so they
/// survive across runs. New options can be added here as the app grows; the
/// INI loader ignores unknown keys and fills missing ones with defaults.
#[derive(Clone, Debug)]
pub struct Settings {
    /// Number of cards to review per day (the "每日数量" control).
    pub daily_limit: usize,
    /// Whether dictation/learn cards auto-play the German word on each card.
    pub speak_on_dictation: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            daily_limit: 20,
            speak_on_dictation: false,
        }
    }
}


