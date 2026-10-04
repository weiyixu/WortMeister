// WortMeister shared core: UI-agnostic vocabulary-trainer logic.
//
// This crate contains the data model, the SM-2 spaced-repetition scheduler,
// answer normalization and the learning-session logic, plus a small C FFI
// (see ffi.rs) so a SwiftUI iOS app (or any C-ABI host) can drive the exact
// same logic as the egui desktop app. There is no UI and no filesystem access
// here on purpose: the host passes data in and gets results back as JSON.

pub mod ffi;
pub mod model;
pub mod session;
pub mod srs;
pub mod util;

#[cfg(test)]
mod tests {
    use crate::model::{CardProgress, ProgressFile};
    use crate::session;
    use crate::srs::{apply_grade, Grade};
    use crate::util::{norm, parse_words};

    const SAMPLE_CSV: &str = "id,level,lesson,german,chinese,example,example_zh,tags,enabled\n\
        A1-1,A1.1,L1,das Haus,房子,Das Haus ist gross.,房子很大。,home,true\n\
        A1-2,A1.1,L2,die Katze,猫,Die Katze schlaeft.,猫在睡觉。,animal,true\n\
        A1-3,A1.2,L1,der Hund,狗,Der Hund laeuft.,狗在跑。,animal,false\n";

    #[test]
    fn parse_skips_disabled_rows() {
        let words = parse_words(SAMPLE_CSV).unwrap();
        // The disabled "der Hund" row must be excluded.
        assert_eq!(words.len(), 2);
        assert_eq!(words[0].german, "das Haus");
    }

    #[test]
    fn norm_folds_umlauts_and_punctuation() {
        // "Fuesse" and the umlaut spelling normalize to the same string.
        assert_eq!(norm("F\u{fc}\u{df}e"), norm("Fuesse"));
        // Punctuation and case and extra spaces are ignored.
        assert_eq!(norm("  Das Haus! "), norm("das haus"));
    }

    #[test]
    fn new_words_are_due_and_queue_respects_limit() {
        let words = parse_words(SAMPLE_CSV).unwrap();
        let progress = ProgressFile::default();
        let queue = session::make_queue(&words, &progress, "all", "all", 1);
        // Both words are new (due), but the daily limit caps the queue at 1.
        assert_eq!(queue.len(), 1);
    }

    #[test]
    fn grade_again_sets_one_day_interval() {
        let mut p = CardProgress::new();
        apply_grade(&mut p, Grade::Again);
        assert_eq!(p.interval_days, 1);
        assert_eq!(p.wrong, 1);
    }
}
