//
//  WortMeisterApp.swift
//  App entry point. Creates the shared Store and shows the main navigation.
//

import SwiftUI

@main
struct WortMeisterApp: App {
    @StateObject private var store = Store()

    var body: some Scene {
        WindowGroup {
            RootView()
                .environmentObject(store)
                .onAppear { store.bootstrap() }
        }
    }
}
