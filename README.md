# Deutsch Worttrainer (Rust / Windows)

A local, offline A1.1/A1.2 German vocabulary learning and dictation app.

## Included
- Daily review queue with configurable daily limit
- Learning cards: German, Chinese meaning, German example and example translation
- Dictation mode with immediate checking, typo-tolerant comparison and feedback
- Spaced repetition grades: Again / Hard / Good / Easy
- Progress dashboard and streak
- Filters by level and lesson
- CSV-based extensible vocabulary store
- JSON progress sidecar, so vocabulary and learner history remain separate
- UTF-8 German/Chinese support

## Build on Windows
1. Install the stable Rust toolchain from rustup.rs.
2. Unzip this project.
3. Run `build_windows.bat`.
4. Start `dist\DeutschWorttrainer.exe`.

At first launch the program searches for `vocabulary.csv` beside the executable, then `data\vocabulary.csv`. Progress is stored as `progress.json` beside the executable.

## CSV schema
Required columns:
`id,level,lesson,german,chinese,example,example_zh,tags,enabled`

Rules:
- UTF-8 CSV with a header row.
- `id` must be unique and stable. Never reuse an old ID for another word.
- `level` examples: `A1.1`, `A1.2`, `A2.1`.
- `lesson` is free text and can follow your textbook structure.
- `tags` uses semicolons, for example `work;radar;technical`.
- `enabled` accepts `true` or `false`.
- Quote fields containing commas or quotation marks according to standard CSV rules.
- Add new rows without changing the program. Existing learning progress remains linked by `id`.

Example:
```csv
id,level,lesson,german,chinese,example,example_zh,tags,enabled
A2-0001,A2.1,Lektion 1,die Messung,测量,"Die Messung ist abgeschlossen.",测量已经完成。,work;technical,true
```

## Data note
The included 1,000-word list is an independently organized CEFR-oriented A1.1/A1.2 starter dataset, not an official Hueber vocabulary index. Example sentences are short learning prompts and can be edited directly in the CSV.
