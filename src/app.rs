// Application state and core (non-UI) logic: loading, saving, filtering,
// building the daily review queue and grading cards.

use std::fs;
use std::path::PathBuf;

use rand::seq::SliceRandom;
use tts::Tts;

use crate::model::{CardProgress, ProgressFile, Word};
use crate::srs::{apply_grade, Grade};
use crate::util::{app_dir, load_progress, load_words, today};

/// The four screens the app can display.
#[derive(Clone, Copy, PartialEq)]
pub enum Mode {
    Home,
    Learn,
    Dictation,
    Browse,
}

/// Central application state, also implements eframe::App in the ui module.
pub struct App {
    pub words: Vec<Word>,
    pub progress: ProgressFile,
    pub progress_path: PathBuf,
    pub mode: Mode,
    pub level: String,
    pub lesson: String,
    pub daily_limit: usize,
    pub queue: Vec<usize>,
    pub pos: usize,
    pub revealed: bool,
    pub input: String,
    pub feedback: Option<bool>,
    pub notice: String,
    pub show_about: bool,
    pub search: String,
    /// Text-to-speech engine, None if the platform backend failed to init.
    pub tts: Option<Tts>,
    /// When true, dictation cards auto-play the German word on each new card.
    pub speak_on_dictation: bool,
    /// Queue position last auto-spoken, to avoid replaying every UI frame.
    pub spoken_pos: Option<usize>,
}

impl App {
    pub fn new() -> Self {
        let base = app_dir();
        let progress_path = base.join("progress.json");
        let (progress, progress_notice) = load_progress(&progress_path);

        let (words, words_notice) = match load_words(&base) {
            Ok(words) => (words, String::new()),
            Err(e) => (Vec::new(), e),
        };

        // Try to initialise the platform text-to-speech backend. On failure we
        // keep running without audio and surface a hint via the notice line.
        let (tts, tts_notice) = match Tts::default() {
            Ok(engine) => (Some(engine), String::new()),
            Err(e) => (
                None,
                format!("语音功能不可用（{e}）。朗读按钮将不起作用。"),
            ),
        };

        // Combine any notices from loading progress and vocabulary.
        let notice = [progress_notice, words_notice, tts_notice]
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("\n");

        Self {
            words,
            progress,
            progress_path,
            mode: Mode::Home,
            level: "全部".into(),
            lesson: "全部".into(),
            daily_limit: 20,
            queue: Vec::new(),
            pos: 0,
            revealed: false,
            input: String::new(),
            feedback: None,
            notice,
            show_about: false,
            search: String::new(),
            tts,
            speak_on_dictation: false,
            spoken_pos: None,
        }
    }

    /// Speak the given text through the platform TTS backend, if available.
    /// Errors are surfaced on the notice line but never panic. Calling this
    /// again while speech is playing interrupts the previous utterance.
    pub fn speak(&mut self, text: &str) {
        if let Some(tts) = self.tts.as_mut() {
            // `interrupt = true` so rapid clicks cancel the prior word.
            if let Err(e) = tts.speak(text, true) {
                self.notice = format!("朗读失败: {e}");
            }
        }
    }

    /// Persist progress to disk, recording a notice on failure.
    pub fn save(&mut self) {
        match serde_json::to_string_pretty(&self.progress) {
            Ok(s) => {
                if let Err(e) = fs::write(&self.progress_path, s) {
                    self.notice = format!("保存失败: {e}");
                }
            }
            Err(e) => self.notice = format!("保存失败: {e}"),
        }
    }

    /// Whether word `i` matches the current level/lesson filters.
    pub fn filtered(&self, i: usize) -> bool {
        let w = &self.words[i];
        (self.level == "全部" || w.level == self.level)
            && (self.lesson == "全部" || w.lesson == self.lesson)
    }

    /// Build the review queue for the given mode: all filtered, due cards,
    /// shuffled and truncated to the daily limit.
    pub fn make_queue(&mut self, mode: Mode) {
        let t = today();
        let mut due: Vec<usize> = (0..self.words.len())
            .filter(|&i| {
                self.filtered(i)
                    && self
                        .progress
                        .cards
                        .get(&self.words[i].id)
                        .map(|p| p.due <= t)
                        .unwrap_or(true)
            })
            .collect();

        due.shuffle(&mut rand::thread_rng());
        due.truncate(self.daily_limit);

        self.queue = due;
        self.pos = 0;
        self.spoken_pos = None;
        self.mode = mode;
        self.revealed = false;
        self.input.clear();
        self.feedback = None;
        self.notice = if self.queue.is_empty() {
            "今天没有到期词条。可以扩大筛选范围，或明天再复习。".into()
        } else {
            String::new()
        };
    }

    /// Grade the current card, advance the queue and persist progress.
    pub fn grade(&mut self, g: Grade) {
        if self.pos >= self.queue.len() {
            return;
        }

        let id = self.words[self.queue[self.pos]].id.clone();
        let p = self
            .progress
            .cards
            .entry(id)
            .or_insert_with(CardProgress::new);
        apply_grade(p, g);

        *self
            .progress
            .daily_counts
            .entry(today().to_string())
            .or_insert(0) += 1;

        self.save();
        self.pos += 1;
        self.revealed = false;
        self.input.clear();
        self.feedback = None;
    }

    /// Distinct levels present in the vocabulary, prefixed with "全部".
    pub fn levels(&self) -> Vec<String> {
        let mut v: Vec<_> = self.words.iter().map(|w| w.level.clone()).collect();
        v.sort();
        v.dedup();
        v.insert(0, "全部".into());
        v
    }

    /// Distinct lessons for the current level, prefixed with "全部".
    pub fn lessons(&self) -> Vec<String> {
        let mut v: Vec<_> = self
            .words
            .iter()
            .filter(|w| self.level == "全部" || w.level == self.level)
            .map(|w| w.lesson.clone())
            .collect();
        v.sort();
        v.dedup();
        v.insert(0, "全部".into());
        v
    }
}
