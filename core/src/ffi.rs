// C FFI boundary used by the SwiftUI iOS app (and any other C-ABI host).
//
// Design: all data crosses the boundary as UTF-8 JSON strings. This keeps the
// ABI tiny and stable (just `const char*` in, `char*` out) and avoids exposing
// complex Rust types or layouts to Swift. Swift encodes/decodes the matching
// Codable structs. Every string returned by the core MUST be released by the
// caller with `wm_string_free` to avoid leaking memory.
//
// Memory contract:
//   - Inputs are borrowed `const char*` (NUL-terminated UTF-8). The core does
//     not take ownership and does not free them.
//   - Outputs are heap-allocated `char*` owned by the caller; free them via
//     `wm_string_free`.

use std::ffi::{c_char, CStr, CString};
use std::panic::{catch_unwind, AssertUnwindSafe};

use serde::{Deserialize, Serialize};

use crate::model::{ProgressFile, Word};
use crate::session;
use crate::srs::Grade;
use crate::util::{norm, parse_words};

/// Convert a borrowed C string into a Rust &str, or return "" on null/invalid.
///
/// # Safety
/// `ptr` must be either null or a valid NUL-terminated UTF-8 C string that
/// remains valid for the duration of the call.
unsafe fn cstr<'a>(ptr: *const c_char) -> &'a str {
    if ptr.is_null() {
        return "";
    }
    CStr::from_ptr(ptr).to_str().unwrap_or("")
}

/// Allocate an owned C string from a Rust String for returning to the caller.
fn out(s: String) -> *mut c_char {
    // Replace interior NULs so CString never fails; JSON never contains them.
    CString::new(s.replace('\0', ""))
        .unwrap_or_default()
        .into_raw()
}

/// A uniform JSON envelope so Swift can always decode `{ ok, data?, error? }`.
#[derive(Serialize)]
struct Envelope<T: Serialize> {
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

fn ok_json<T: Serialize>(data: T) -> *mut c_char {
    out(serde_json::to_string(&Envelope {
        ok: true,
        data: Some(data),
        error: None,
    })
    .unwrap_or_else(|e| format!("{{\"ok\":false,\"error\":\"serialize: {e}\"}}")))
}

fn err_json(msg: impl Into<String>) -> *mut c_char {
    out(serde_json::to_string(&Envelope::<()> {
        ok: false,
        data: None,
        error: Some(msg.into()),
    })
    .unwrap_or_else(|_| "{\"ok\":false,\"error\":\"unknown\"}".into()))
}

/// Run `f` catching panics so a bug in the core can never unwind across the
/// FFI boundary (which is undefined behaviour); return a JSON error instead.
fn guard(f: impl FnOnce() -> *mut c_char) -> *mut c_char {
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(ptr) => ptr,
        Err(_) => err_json("internal panic in core"),
    }
}

// ---------------------------------------------------------------------------
// Exported C functions
// ---------------------------------------------------------------------------

/// Return the core library version string as JSON `{ ok, data }`.
#[no_mangle]
pub extern "C" fn wm_version() -> *mut c_char {
    ok_json(env!("CARGO_PKG_VERSION"))
}

/// Parse vocabulary CSV text, returning `{ ok, data: [Word] }` or an error.
///
/// # Safety
/// `csv_ptr` must be null or a valid NUL-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn wm_parse_words(csv_ptr: *const c_char) -> *mut c_char {
    guard(|| {
        let csv = cstr(csv_ptr);
        match parse_words(csv) {
            Ok(words) => ok_json(words),
            Err(e) => err_json(e),
        }
    })
}

/// Arguments for building a review queue, decoded from JSON.
#[derive(Deserialize)]
struct QueueArgs {
    words: Vec<Word>,
    #[serde(default)]
    progress: ProgressFile,
    #[serde(default = "all")]
    level: String,
    #[serde(default = "all")]
    lesson: String,
    #[serde(default = "default_limit")]
    daily_limit: usize,
}

fn all() -> String {
    "all".into()
}
fn default_limit() -> usize {
    20
}

/// Build a review queue. Input JSON: `{ words, progress, level, lesson,
/// daily_limit }`. Output: `{ ok, data: [word_id] }`.
///
/// # Safety
/// `args_ptr` must be null or a valid NUL-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn wm_make_queue(args_ptr: *const c_char) -> *mut c_char {
    guard(|| {
        let args: QueueArgs = match serde_json::from_str(cstr(args_ptr)) {
            Ok(a) => a,
            Err(e) => return err_json(format!("bad args: {e}")),
        };
        let queue = session::make_queue(
            &args.words,
            &args.progress,
            &args.level,
            &args.lesson,
            args.daily_limit,
        );
        ok_json(queue)
    })
}

/// Arguments for grading a card, decoded from JSON.
#[derive(Deserialize)]
struct GradeArgs {
    progress: ProgressFile,
    word_id: String,
    grade: Grade,
}

/// Apply a grade to a card. Input JSON: `{ progress, word_id, grade }` where
/// grade is one of "again"|"hard"|"good"|"easy". Output: `{ ok, data:
/// ProgressFile }` with the updated progress for the host to persist.
///
/// # Safety
/// `args_ptr` must be null or a valid NUL-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn wm_grade_card(args_ptr: *const c_char) -> *mut c_char {
    guard(|| {
        let mut args: GradeArgs = match serde_json::from_str(cstr(args_ptr)) {
            Ok(a) => a,
            Err(e) => return err_json(format!("bad args: {e}")),
        };
        session::grade_card(&mut args.progress, &args.word_id, args.grade);
        ok_json(args.progress)
    })
}

/// Check a dictation answer leniently. Input JSON: `{ expected, actual }`.
/// Output: `{ ok, data: bool }` true when the normalized forms are equal.
///
/// # Safety
/// `args_ptr` must be null or a valid NUL-terminated UTF-8 C string.
#[derive(Deserialize)]
struct CheckArgs {
    expected: String,
    actual: String,
}

#[no_mangle]
pub unsafe extern "C" fn wm_check_answer(args_ptr: *const c_char) -> *mut c_char {
    guard(|| {
        let args: CheckArgs = match serde_json::from_str(cstr(args_ptr)) {
            Ok(a) => a,
            Err(e) => return err_json(format!("bad args: {e}")),
        };
        ok_json(norm(&args.expected) == norm(&args.actual))
    })
}

/// Return distinct levels for the given words. Input JSON: `[Word]`.
/// Output: `{ ok, data: [String] }`.
///
/// # Safety
/// `words_ptr` must be null or a valid NUL-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn wm_levels(words_ptr: *const c_char) -> *mut c_char {
    guard(|| match serde_json::from_str::<Vec<Word>>(cstr(words_ptr)) {
        Ok(words) => ok_json(session::levels(&words)),
        Err(e) => err_json(format!("bad args: {e}")),
    })
}

/// Free a string previously returned by any `wm_*` function.
///
/// # Safety
/// `ptr` must be null or a pointer previously returned by this library and not
/// already freed. Passing any other pointer is undefined behaviour.
#[no_mangle]
pub unsafe extern "C" fn wm_string_free(ptr: *mut c_char) {
    if !ptr.is_null() {
        drop(CString::from_raw(ptr));
    }
}
