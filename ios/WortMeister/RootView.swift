//
//  RootView.swift
//  Home screen: pick a level + daily limit, start a learn or dictation session,
//  and jump into the review flow. Mirrors the desktop app's Home mode.
//

import SwiftUI

struct RootView: View {
    @EnvironmentObject var store: Store
    @State private var mode: ReviewView.Mode = .learn

    var body: some View {
        NavigationStack {
            Form {
                Section("Filter") {
                    Picker("Level", selection: $store.level) {
                        ForEach(store.levels, id: \.self) { lvl in
                            Text(lvl == "all" ? "All" : lvl).tag(lvl)
                        }
                    }
                    Stepper("Daily limit: \(store.dailyLimit)",
                            value: $store.dailyLimit, in: 5...100, step: 5)
                }

                Section("Study") {
                    NavigationLink {
                        startAndShow(.learn)
                    } label: {
                        Label("Learn cards", systemImage: "rectangle.on.rectangle")
                    }
                    NavigationLink {
                        startAndShow(.dictation)
                    } label: {
                        Label("Dictation", systemImage: "pencil.and.scribble")
                    }
                }

                if !store.notice.isEmpty {
                    Section("Notice") {
                        Text(store.notice).foregroundStyle(.secondary)
                    }
                }

                Section {
                    Text("\(store.words.count) words loaded")
                        .foregroundStyle(.secondary)
                }
            }
            .navigationTitle("WortMeister")
        }
    }

    /// Build the queue for `mode` then present the review view.
    private func startAndShow(_ mode: ReviewView.Mode) -> some View {
        store.startSession()
        return ReviewView(mode: mode)
    }
}
