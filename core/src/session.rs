// UI-agnostic learning session logic: build the due-card queue from words +
// progress, and apply a grade to a card. The desktop app's App struct and the
// iOS SwiftUI view both drive these same functions, so scheduling behaviour is
// identical across platforms.

use rand::seq::SliceRandom;
use serde::Serialize;

use crate::model::{CardProgress, ProgressFile, Word};
use crate::srs::{apply_grade, Grade};
use crate::util::today;


/// Whether word `w` matches the given level/lesson filters.
/// The sentinel value "all" (any case) means "no filter on this field".
fn filtered(w: &Word, level: &str, lesson: &str) -> bool {
    let level_ok = level.eq_ignore_ascii_case("all") || w.level == level;
    let lesson_ok = lesson.eq_ignore_ascii_case("all") || w.lesson == lesson;
    level_ok && lesson_ok
}

/// Build a review queue: all filtered, due cards, shuffled and truncated to
/// `daily_limit`. Returns the list of word IDs to review in order.
///
/// A card is "due" when it has no progress yet (new) or its due date is today
/// or earlier.
pub fn make_queue(
    words: &[Word],
    progress: &ProgressFile,
    level: &str,
    lesson: &str,
    daily_limit: usize,
) -> Vec<String> {
    let t = today();
    let mut due: Vec<String> = words
        .iter()
        .filter(|w| {
            filtered(w, level, lesson)
                && progress
                    .cards
                    .get(&w.id)
                    .map(|p| p.due <= t)
                    .unwrap_or(true)
        })
        .map(|w| w.id.clone())
        .collect();

    due.shuffle(&mut rand::thread_rng());
    due.truncate(daily_limit);
    due
}

/// Apply a grade to the card with `word_id`, mutating `progress` in place and
/// incrementing today's daily count. Mirrors the desktop App::grade logic.
pub fn grade_card(progress: &mut ProgressFile, word_id: &str, g: Grade) {
    let p = progress
        .cards
        .entry(word_id.to_string())
        .or_insert_with(CardProgress::new);
    apply_grade(p, g);

    *progress
        .daily_counts
        .entry(today().to_string())
        .or_insert(0) += 1;
}

/// Distinct levels present in the vocabulary, sorted, prefixed with "all".
pub fn levels(words: &[Word]) -> Vec<String> {
    let mut v: Vec<String> = words.iter().map(|w| w.level.clone()).collect();
    v.sort();
    v.dedup();
    v.insert(0, "all".into());
    v
}

/// Aggregate learning statistics computed from the vocabulary and progress.
/// Serialized to JSON for the UI (desktop dashboard, PWA stats screen, etc.).
#[derive(Serialize)]
pub struct Stats {
    /// Total words in the (unfiltered) vocabulary.
    pub total_words: usize,
    /// Cards that have at least one review recorded ("started").
    pub studied: usize,
    /// Cards never reviewed yet.
    pub new: usize,
    /// Cards due today or earlier (new cards count as due).
    pub due_today: usize,
    /// Cards considered "learned": at least 3 successful repetitions.
    pub learned: usize,
    /// Sum of all correct answers across cards.
    pub total_correct: u32,
    /// Sum of all wrong answers across cards.
    pub total_wrong: u32,
    /// Overall accuracy percentage (0-100), 0 when no answers yet.
    pub accuracy: u32,
    /// Number of reviews done today (from daily_counts).
    pub reviews_today: u32,
    /// Consecutive-day streak ending today (or yesterday) with >=1 review.
    pub streak: u32,
}

/// A card is treated as "learned" once it has this many repetitions.
const LEARNED_REPS: u32 = 3;

/// Compute aggregate statistics for the full vocabulary and progress.
pub fn stats(words: &[Word], progress: &ProgressFile) -> Stats {
    let t = today();

    let total_words = words.len();
    let mut studied = 0usize;
    let mut due_today = 0usize;
    let mut learned = 0usize;
    let mut total_correct = 0u32;
    let mut total_wrong = 0u32;

    for w in words {
        match progress.cards.get(&w.id) {
            Some(p) => {
                studied += 1;
                total_correct += p.correct;
                total_wrong += p.wrong;
                if p.due <= t {
                    due_today += 1;
                }
                if p.repetitions >= LEARNED_REPS {
                    learned += 1;
                }
            }
            // No progress yet: a brand-new card is always due.
            None => due_today += 1,
        }
    }

    let new = total_words.saturating_sub(studied);
    let answered = total_correct + total_wrong;
    let accuracy = if answered > 0 {
        ((total_correct as f64 / answered as f64) * 100.0).round() as u32
    } else {
        0
    };

    let reviews_today = progress
        .daily_counts
        .get(&t.to_string())
        .copied()
        .unwrap_or(0);

    Stats {
        total_words,
        studied,
        new,
        due_today,
        learned,
        total_correct,
        total_wrong,
        accuracy,
        reviews_today,
        streak: streak(progress),
    }
}

/// Count the consecutive-day streak of days with at least one review, ending
/// today or yesterday. If the user reviewed today or yesterday, we walk back
/// day by day while daily_counts has a positive entry.
fn streak(progress: &ProgressFile) -> u32 {
    use chrono::Duration;

    let has = |d: chrono::NaiveDate| {
        progress
            .daily_counts
            .get(&d.to_string())
            .copied()
            .unwrap_or(0)
            > 0
    };

    let t = today();
    // Anchor the streak at today if reviewed today, else yesterday, else 0.
    let mut day = if has(t) {
        t
    } else if has(t - Duration::days(1)) {
        t - Duration::days(1)
    } else {
        return 0;
    };

    let mut count = 0u32;
    while has(day) {
        count += 1;
        day -= Duration::days(1);
    }
    count
}


/// Distinct lessons for `level` (or all levels when "all"), prefixed "all".
pub fn lessons(words: &[Word], level: &str) -> Vec<String> {
    let mut v: Vec<String> = words
        .iter()
        .filter(|w| level.eq_ignore_ascii_case("all") || w.level == level)
        .map(|w| w.lesson.clone())
        .collect();
    v.sort();
    v.dedup();
    v.insert(0, "all".into());
    v
}
