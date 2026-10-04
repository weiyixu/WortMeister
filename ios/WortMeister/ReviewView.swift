//
//  ReviewView.swift
//  The card review flow for both Learn and Dictation modes. Learn shows the
//  German word and lets you reveal the meaning, then grade with SM-2 buttons.
//  Dictation hides the word, you type it, and the Rust core checks leniently.
//

import SwiftUI

struct ReviewView: View {
    enum Mode { case learn, dictation }

    @EnvironmentObject var store: Store
    let mode: Mode

    @State private var typed = ""
    @State private var lastResult: Bool?

    var body: some View {
        VStack(spacing: 24) {
            if let word = store.currentWord {
                progressHeader
                card(for: word)
                Spacer()
                controls(for: word)
            } else {
                finished
            }
        }
        .padding()
        .navigationTitle(mode == .learn ? "Learn" : "Dictation")
        .navigationBarTitleDisplayMode(.inline)
    }

    // MARK: - Pieces

    private var progressHeader: some View {
        Text("\(store.pos + 1) / \(store.queue.count)")
            .font(.subheadline)
            .foregroundStyle(.secondary)
    }

    @ViewBuilder
    private func card(for word: Word) -> some View {
        VStack(spacing: 16) {
            if mode == .learn || store.revealed {
                Text(word.german)
                    .font(.largeTitle).bold()
                    .multilineTextAlignment(.center)
                Button {
                    store.speak(word.german)
                } label: {
                    Label("Play", systemImage: "speaker.wave.2.fill")
                }
            } else {
                Text("Type what you hear / see")
                    .font(.title3)
                    .foregroundStyle(.secondary)
                Button {
                    store.speak(word.german)
                } label: {
                    Label("Play", systemImage: "speaker.wave.2.fill")
                }
            }

            if store.revealed {
                Divider()
                Text(word.chinese).font(.title2)
                Text(word.example).italic()
                Text(word.example_zh).foregroundStyle(.secondary)
            }
        }
        .frame(maxWidth: .infinity)
        .padding()
        .background(.quaternary, in: RoundedRectangle(cornerRadius: 16))
    }

    @ViewBuilder
    private func controls(for word: Word) -> some View {
        if mode == .dictation && !store.revealed {
            TextField("Your answer", text: $typed)
                .textFieldStyle(.roundedBorder)
                .autocorrectionDisabled()
                .textInputAutocapitalization(.never)
            if let r = lastResult {
                Text(r ? "Correct" : "Not quite")
                    .foregroundStyle(r ? .green : .red)
            }
            Button("Check") {
                lastResult = store.checkDictation(typed)
                store.revealed = true
            }
            .buttonStyle(.borderedProminent)
            .disabled(typed.isEmpty)
        } else if !store.revealed {
            Button("Show answer") { store.revealed = true }
                .buttonStyle(.borderedProminent)
        } else {
            gradeButtons
        }
    }

    private var gradeButtons: some View {
        HStack(spacing: 10) {
            gradeButton("Again", .again, .red)
            gradeButton("Hard", .hard, .orange)
            gradeButton("Good", .good, .blue)
            gradeButton("Easy", .easy, .green)
        }
    }

    private func gradeButton(_ title: String, _ g: Grade, _ color: Color) -> some View {
        Button(title) {
            store.grade(g)
            typed = ""
            lastResult = nil
        }
        .frame(maxWidth: .infinity)
        .tint(color)
        .buttonStyle(.bordered)
    }

    private var finished: some View {
        VStack(spacing: 12) {
            Image(systemName: "checkmark.seal.fill")
                .font(.system(size: 56))
                .foregroundStyle(.green)
            Text("Session complete")
                .font(.title2).bold()
            if !store.notice.isEmpty {
                Text(store.notice).foregroundStyle(.secondary)
            }
        }
    }
}
