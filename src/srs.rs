// Spaced-repetition scheduling. Applies a review grade to a CardProgress and
// computes the next interval and due date. Behavior matches the original
// single-file implementation exactly.

use chrono::Duration;

use crate::model::CardProgress;
use crate::util::today;

/// The four review grades presented after a card is revealed.
#[derive(Clone, Copy)]
pub enum Grade {
    Again,
    Hard,
    Good,
    Easy,
}

/// Update a card's scheduling state for the given grade.
pub fn apply_grade(p: &mut CardProgress, g: Grade) {
    match g {
        Grade::Again => {
            p.repetitions = 0;
            p.interval_days = 1;
            p.ease = (p.ease - 0.2).max(1.3);
            p.wrong += 1;
        }
        Grade::Hard => {
            p.repetitions += 1;
            p.interval_days = (p.interval_days.max(1) as f32 * 1.2).round() as i64;
            p.ease = (p.ease - 0.15).max(1.3);
            p.correct += 1;
        }
        Grade::Good => {
            p.repetitions += 1;
            p.interval_days = if p.repetitions == 1 {
                1
            } else if p.repetitions == 2 {
                3
            } else {
                (p.interval_days as f32 * p.ease).round().max(1.0) as i64
            };
            p.correct += 1;
        }
        Grade::Easy => {
            p.repetitions += 1;
            p.ease = (p.ease + 0.15).min(3.2);
            p.interval_days = if p.repetitions <= 1 {
                4
            } else {
                (p.interval_days as f32 * p.ease * 1.3).round().max(4.0) as i64
            };
            p.correct += 1;
        }
    }

    p.last_review = Some(today());
    p.due = today() + Duration::days(p.interval_days);
}
