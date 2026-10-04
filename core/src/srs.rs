// Spaced-repetition scheduling based on the SM-2 algorithm. Applies a review
// grade to a CardProgress and computes the next interval and due date.
//
// SM-2 reference intervals for a successful review:
//   repetition 1 -> 1 day
//   repetition 2 -> 6 days
//   repetition n -> round(previous_interval * ease)
// The ease factor is bounded to a sane range [1.3, 3.2].

use chrono::Duration;
use serde::{Deserialize, Serialize};

use crate::model::CardProgress;
use crate::util::today;

/// Lower and upper bounds for the ease factor.
const EASE_MIN: f32 = 1.3;
const EASE_MAX: f32 = 3.2;

/// The four review grades presented after a card is revealed.
/// Serialize/Deserialize so Swift can send a grade across the FFI as JSON.
#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Grade {
    Again,
    Hard,
    Good,
    Easy,
}

/// Update a card's scheduling state for the given grade.
pub fn apply_grade(p: &mut CardProgress, g: Grade) {
    match g {
        // Lapse: reset the streak, review again tomorrow, reduce ease.
        Grade::Again => {
            p.repetitions = 0;
            p.interval_days = 1;
            p.ease = (p.ease - 0.2).max(EASE_MIN);
            p.wrong += 1;
        }
        // Recalled with difficulty: small growth, reduce ease slightly.
        Grade::Hard => {
            p.repetitions += 1;
            p.ease = (p.ease - 0.15).max(EASE_MIN);
            p.interval_days = next_interval(p, 1.2);
            p.correct += 1;
        }
        // Recalled correctly: standard SM-2 growth.
        Grade::Good => {
            p.repetitions += 1;
            p.interval_days = next_interval(p, 1.0);
            p.correct += 1;
        }
        // Recalled easily: increase ease and grow faster.
        Grade::Easy => {
            p.repetitions += 1;
            p.ease = (p.ease + 0.15).min(EASE_MAX);
            p.interval_days = next_interval(p, 1.3);
            p.correct += 1;
        }
    }

    p.last_review = Some(today());
    p.due = today() + Duration::days(p.interval_days);
}

/// Compute the next interval for a successful review following SM-2, applying
/// an extra `factor` multiplier for the Hard/Easy variants.
///
/// Assumes `p.repetitions` has already been incremented for this review.
fn next_interval(p: &CardProgress, factor: f32) -> i64 {
    let base = match p.repetitions {
        0 | 1 => 1.0,
        2 => 6.0,
        _ => p.interval_days.max(1) as f32 * p.ease,
    };
    (base * factor).round().max(1.0) as i64
}
