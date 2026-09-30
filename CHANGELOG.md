# Changelog

All notable changes to Deutsch Worttrainer are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
