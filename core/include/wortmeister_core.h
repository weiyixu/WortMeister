/*
 * wortmeister_core.h - C API for the WortMeister shared Rust core.
 *
 * All functions take and return UTF-8, NUL-terminated C strings. Inputs are
 * borrowed (the core never frees them). Every non-null char* returned by a
 * wm_* function is heap-allocated and owned by the caller: release it with
 * wm_string_free exactly once.
 *
 * Returned strings are JSON with the shape:
 *   { "ok": true,  "data": <result> }
 *   { "ok": false, "error": "<message>" }
 *
 * This header is hand-written to match src/ffi.rs. If you prefer generated
 * headers, add cbindgen (see IOS_PLAN.md) and regenerate during the build.
 */

#ifndef WORTMEISTER_CORE_H
#define WORTMEISTER_CORE_H

#ifdef __cplusplus
extern "C" {
#endif

/* Core library version, e.g. {"ok":true,"data":"0.0.9"}. */
char *wm_version(void);

/* Parse vocabulary CSV text -> {"ok":true,"data":[Word,...]}. */
char *wm_parse_words(const char *csv);

/* Build a review queue.
 * Input JSON: {"words":[Word],"progress":ProgressFile,"level":"all",
 *              "lesson":"all","daily_limit":20}
 * Output: {"ok":true,"data":["word_id",...]} */
char *wm_make_queue(const char *args_json);

/* Apply a grade to a card.
 * Input JSON: {"progress":ProgressFile,"word_id":"A1-1","grade":"good"}
 *   grade is one of "again" | "hard" | "good" | "easy".
 * Output: {"ok":true,"data":ProgressFile}  (updated; persist on the host) */
char *wm_grade_card(const char *args_json);

/* Lenient dictation check.
 * Input JSON: {"expected":"Fuesse","actual":"...."}
 * Output: {"ok":true,"data":true|false} */
char *wm_check_answer(const char *args_json);

/* Distinct levels for a word list.
 * Input JSON: [Word,...]  Output: {"ok":true,"data":["all","A1.1",...]} */
char *wm_levels(const char *words_json);

/* Free a string returned by any wm_* function. Safe to pass NULL. */
void wm_string_free(char *ptr);

#ifdef __cplusplus
}
#endif

#endif /* WORTMEISTER_CORE_H */
