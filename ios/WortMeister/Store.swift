//
//  Store.swift
//  Observable app state for the SwiftUI UI. Holds the vocabulary + progress,
//  drives the Rust core for queue building / grading / dictation checking,
//  and persists progress to the app sandbox (Documents directory).
//
//  iOS sandbox note: unlike the desktop app (which reads/writes files next to
//  the executable), iOS apps may only write inside their container. We bundle
//  vocabulary.csv as a read-only resource and store progress.json in
//  Documents.
//

import Foundation
import AVFoundation

@MainActor
final class Store: ObservableObject {
    @Published var words: [Word] = []
    @Published var progress: ProgressFile = .empty
    @Published var levels: [String] = ["all"]

    @Published var level: String = "all"
    @Published var dailyLimit: Int = 20

    @Published var queue: [String] = []    // word IDs for today's session
    @Published var pos: Int = 0
    @Published var revealed = false
    @Published var notice: String = ""

    private let speech = AVSpeechSynthesizer()

    private var progressURL: URL {
        let docs = FileManager.default.urls(
            for: .documentDirectory, in: .userDomainMask)[0]
        return docs.appendingPathComponent("progress.json")
    }

    /// Current card's word, or nil when the queue is finished.
    var currentWord: Word? {
        guard pos < queue.count else { return nil }
        let id = queue[pos]
        return words.first { $0.id == id }
    }

    // MARK: - Lifecycle

    /// Load bundled vocabulary and any saved progress. Call once at launch.
    func bootstrap() {
        loadProgress()
        loadWords()
    }

    private func loadWords() {
        guard let url = Bundle.main.url(
                forResource: "vocabulary", withExtension: "csv"),
              let csv = try? String(contentsOf: url, encoding: .utf8)
        else {
            notice = "vocabulary.csv not found in app bundle."
            return
        }
        do {
            words = try WortMeisterCore.parseWords(csv: csv)
            levels = try WortMeisterCore.levels(words: words)
        } catch {
            notice = error.localizedDescription
        }
    }

    private func loadProgress() {
        guard let data = try? Data(contentsOf: progressURL) else { return }
        if let p = try? JSONDecoder().decode(ProgressFile.self, from: data) {
            progress = p
        }
    }

    private func saveProgress() {
        if let data = try? JSONEncoder().encode(progress) {
            try? data.write(to: progressURL, options: .atomic)
        }
    }

    // MARK: - Session

    /// Build today's queue from the current filter and daily limit.
    func startSession() {
        do {
            queue = try WortMeisterCore.makeQueue(
                words: words, progress: progress,
                level: level, dailyLimit: dailyLimit)
            pos = 0
            revealed = false
            notice = queue.isEmpty
                ? "No cards due today. Widen the filter or come back tomorrow."
                : ""
        } catch {
            notice = error.localizedDescription
        }
    }

    /// Grade the current card, persist progress and advance.
    func grade(_ g: Grade) {
        guard let word = currentWord else { return }
        do {
            progress = try WortMeisterCore.grade(
                progress: progress, wordID: word.id, grade: g)
            saveProgress()
        } catch {
            notice = error.localizedDescription
        }
        pos += 1
        revealed = false
    }

    /// Lenient dictation check against the current card's German word.
    func checkDictation(_ typed: String) -> Bool {
        guard let word = currentWord else { return false }
        return (try? WortMeisterCore.checkAnswer(
            expected: word.german, actual: typed)) ?? false
    }

    // MARK: - Speech (native iOS TTS, replaces the desktop `tts` crate)

    /// Speak the given German text using AVSpeechSynthesizer with a de-DE voice.
    func speak(_ text: String) {
        let utterance = AVSpeechUtterance(string: text)
        utterance.voice = AVSpeechSynthesisVoice(language: "de-DE")
        speech.stopSpeaking(at: .immediate)
        speech.speak(utterance)
    }
}
