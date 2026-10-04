//
//  WortMeisterCore.swift
//  Swift wrapper around the Rust core C FFI (wortmeister_core.h).
//
//  The Rust core does all vocabulary/scheduling logic. Here we only marshal
//  Codable Swift structs to/from the JSON strings the C API exchanges, and
//  make sure every char* returned by the core is freed with wm_string_free.
//

import Foundation

// MARK: - Data models (must match src/model.rs field names)

struct Word: Codable, Identifiable, Hashable {
    let id: String
    let level: String
    let lesson: String
    let german: String
    let chinese: String
    let example: String
    let example_zh: String
    let tags: String
    let enabled: Bool
}

struct CardProgress: Codable {
    var repetitions: UInt32
    var interval_days: Int64
    var ease: Float
    var due: String            // ISO date "YYYY-MM-DD" (chrono NaiveDate)
    var correct: UInt32
    var wrong: UInt32
    var last_review: String?
}

struct ProgressFile: Codable {
    var cards: [String: CardProgress]
    var daily_counts: [String: UInt32]

    static let empty = ProgressFile(cards: [:], daily_counts: [:])
}

enum Grade: String, Codable {
    case again, hard, good, easy
}

// MARK: - FFI envelope

/// The uniform { ok, data?, error? } wrapper the core returns.
private struct Envelope<T: Decodable>: Decodable {
    let ok: Bool
    let data: T?
    let error: String?
}

enum CoreError: Error, LocalizedError {
    case core(String)
    case decode(String)

    var errorDescription: String? {
        switch self {
        case .core(let m): return m
        case .decode(let m): return "decode: \(m)"
        }
    }
}

// MARK: - Core facade

/// Thin, safe Swift API over the Rust core. All methods are synchronous and
/// cheap (pure computation), so they can be called from the main actor.
enum WortMeisterCore {

    /// Call a wm_* function that takes one C string and returns a JSON envelope,
    /// decoding `data` as `T`. Guarantees the returned C string is freed.
    private static func call<T: Decodable>(
        _ fn: (UnsafePointer<CChar>?) -> UnsafeMutablePointer<CChar>?,
        _ input: String,
        as _: T.Type
    ) throws -> T {
        let raw: UnsafeMutablePointer<CChar>? = input.withCString { fn($0) }
        guard let raw else { throw CoreError.core("null result from core") }
        defer { wm_string_free(raw) }

        let json = String(cString: raw)
        guard let bytes = json.data(using: .utf8) else {
            throw CoreError.decode("non-utf8 result")
        }
        let env: Envelope<T>
        do {
            env = try JSONDecoder().decode(Envelope<T>.self, from: bytes)
        } catch {
            throw CoreError.decode("\(error) (payload: \(json))")
        }
        guard env.ok, let data = env.data else {
            throw CoreError.core(env.error ?? "unknown core error")
        }
        return data
    }

    /// Core library version string.
    static func version() throws -> String {
        let raw = wm_version()
        guard let raw else { throw CoreError.core("null version") }
        defer { wm_string_free(raw) }
        let json = String(cString: raw)
        let env = try JSONDecoder().decode(
            Envelope<String>.self, from: Data(json.utf8))
        guard env.ok, let v = env.data else {
            throw CoreError.core(env.error ?? "unknown")
        }
        return v
    }

    /// Parse bundled vocabulary.csv text into Word rows.
    static func parseWords(csv: String) throws -> [Word] {
        try call(wm_parse_words, csv, as: [Word].self)
    }

    /// Build today's review queue (list of word IDs) from words + progress.
    static func makeQueue(
        words: [Word],
        progress: ProgressFile,
        level: String = "all",
        lesson: String = "all",
        dailyLimit: Int = 20
    ) throws -> [String] {
        struct Args: Encodable {
            let words: [Word]
            let progress: ProgressFile
            let level: String
            let lesson: String
            let daily_limit: Int
        }
        let args = Args(words: words, progress: progress,
                        level: level, lesson: lesson, daily_limit: dailyLimit)
        let json = String(data: try JSONEncoder().encode(args), encoding: .utf8)!
        return try call(wm_make_queue, json, as: [String].self)
    }

    /// Apply a grade to a card and return the updated progress to persist.
    static func grade(
        progress: ProgressFile, wordID: String, grade: Grade
    ) throws -> ProgressFile {
        struct Args: Encodable {
            let progress: ProgressFile
            let word_id: String
            let grade: Grade
        }
        let args = Args(progress: progress, word_id: wordID, grade: grade)
        let json = String(data: try JSONEncoder().encode(args), encoding: .utf8)!
        return try call(wm_grade_card, json, as: ProgressFile.self)
    }

    /// Lenient dictation check (umlaut/case/punctuation tolerant).
    static func checkAnswer(expected: String, actual: String) throws -> Bool {
        struct Args: Encodable { let expected: String; let actual: String }
        let args = Args(expected: expected, actual: actual)
        let json = String(data: try JSONEncoder().encode(args), encoding: .utf8)!
        return try call(wm_check_answer, json, as: Bool.self)
    }

    /// Distinct levels present in the word list (prefixed with "all").
    static func levels(words: [Word]) throws -> [String] {
        let json = String(data: try JSONEncoder().encode(words), encoding: .utf8)!
        return try call(wm_levels, json, as: [String].self)
    }
}
