# Changelog

All notable changes to Deutsch Worttrainer are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.0.7] - 2026-10-01

### Added
- Text-to-speech pronunciation using the platform speech backend (Windows
  SAPI), fully offline. A "🔊 播放读音" button on the learning card and the
  dictation card plays the German word on demand.
- Auto-play option on both cards: dictation shows "出题时自动播放读音" and
  learning shows "显示单词时自动播放读音". When enabled, each new card speaks
  the German word once (not on every UI frame). The two modes share one
  preference toggle.
- If the speech backend fails to initialise, a notice is shown and the app
  keeps working without audio. For accurate German pronunciation a German
  speech voice must be installed in Windows.

### Changed
- Text-to-speech now selects an installed German voice at startup, so words
  are pronounced in German instead of the default system voice (often English).
  If no German voice is installed, a notice explains how to add one via Windows
  language settings and playback falls back to the default voice.
- Home dashboard statistics reworked into three mutually exclusive buckets,
  all scoped to the current level / lesson filter: 未学新词 (no record yet),
  今日待复习 (studied and due today), 已掌握 (studied, scheduled for a future
  date). The three now add up to the filtered vocabulary count. The previous
  "已学习" / "今日到期" figures used inconsistent scopes (whole library vs.
  filtered) and could overlap, so they did not sum to the total.

## [0.0.6] - 2026-10-01

### Added
- Learning mode: pressing Enter now reveals the meaning and example, the same
  as clicking the "显示释义与例句" button.

### Changed
- Dictation mode: the answer input field now automatically receives keyboard
  focus so the user can start typing immediately without clicking it first.

## [0.0.5] - 2026-10-01

### Added
- Dictation now tolerates ASCII transcriptions of German umlauts and eszett,
  so `ue/ae/oe/ss` are accepted for `ü/ä/ö/ß` (e.g. `Fuesse` matches `Füße`).
- Keyboard shortcuts 1 / 2 / 3 / 4 in the review card to grade Again / Hard /
  Good / Easy without using the mouse; button labels show the shortcut.
- Search box in the vocabulary browser that filters by German, Chinese or tag.
- Learning statistics on the home page: overall accuracy (correct / wrong /
  percentage) and a 7-day review-count bar chart.

## [0.0.4] - 2026-09-30

### Added
- "帮助" (Help) menu in the top bar with an "关于" (About) item.
- About window showing the application name, version and a short description.

## [0.0.3] - 2026-09-30

### Fixed
- Corrected the SM-2 spaced-repetition scheduling. A successful review now
  follows the standard intervals (1 day, then 6 days, then previous interval
  multiplied by the ease factor) instead of the previous incorrect 3-day step.

### Changed
- Extracted the interval calculation into a shared `next_interval()` helper so
  Hard / Good / Easy grades reuse one code path via a factor multiplier.
- Introduced named constants `EASE_MIN` and `EASE_MAX` for the ease bounds.

### Added
- Corruption protection for `progress.json`: if the file exists but cannot be
  parsed, it is backed up to a timestamped `.bak` file and a notice is shown,
  instead of silently discarding all learning history.

## [0.0.2] - 2026-09-30

### Changed
- Refactored the single monolithic `main.rs` into focused modules for
  readability and maintainability:
  - `model.rs` - data types (`Word`, `CardProgress`, `ProgressFile`)
  - `util.rs` - date, text normalization, paths and CSV loading helpers
  - `srs.rs` - spaced-repetition scheduling (`Grade`, `apply_grade`)
  - `app.rs` - application state and core logic
  - `ui.rs` - all egui rendering
  - `main.rs` - entry point and font/window setup
- Reformatted all code with proper indentation and documentation comments.

### Fixed
- Removed a `useless_conversion` clippy warning in the font setup code.
