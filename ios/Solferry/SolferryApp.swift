import PhotosUI
import SwiftUI

private let presenceEverySeconds = 20.0

enum Route: Hashable {
    case addDevice, settings
}

@main
struct SolferryApp: App {
    @State private var engine = Engine()

    var body: some Scene {
        WindowGroup {
            RootView()
                .environment(engine)
                .themed()
        }
    }
}

private struct RootView: View {
    @Environment(Engine.self) private var engine
    @Environment(\.scenePhase) private var phase
    @State private var path: [Route] = []
    @State private var staged: [URL] = []
    @State private var target: PairedDevice?
    @State private var choosing = false
    @State private var browsing = false
    @State private var browsingPhotos = false
    @State private var photos: [PhotosPickerItem] = []

    var body: some View {
        NavigationStack(path: $path) {
            HomeScreen(staged: $staged, pick: pick)
                .navigationDestination(for: Route.self) { route in
                    switch route {
                    case .addDevice: AddDeviceScreen()
                    case .settings: SettingsScreen()
                    }
                }
        }
        .confirmationDialog("Send", isPresented: $choosing) {
            Button("Photos and videos") { browsingPhotos = true }
            Button("Files") { browsing = true }
        }
        .fileImporter(isPresented: $browsing, allowedContentTypes: [.item], allowsMultipleSelection: true) { result in
            if case let .success(urls) = result { deliver(urls) }
        }
        .photosPicker(isPresented: $browsingPhotos, selection: $photos, matching: nil)
        .onChange(of: photos) { _, picked in loadPhotos(picked) }
        .onOpenURL { url in staged = [url] }
        .onChange(of: staged) { _, files in if !files.isEmpty { path = [] } }
        .onChange(of: engine.state.pairedCount) { path = [] }
        .onChange(of: engine.state.asking) { before, now in if now > before { path = [] } }
        .onChange(of: phase, initial: true) { _, now in engine.sceneChanged(active: now == .active) }
        .task(id: phase) {
            while phase == .active, !Task.isCancelled {
                engine.bridge.checkPresence()
                try? await Task.sleep(for: .seconds(presenceEverySeconds))
            }
        }
    }

    private func pick(_ device: PairedDevice?) {
        target = device
        choosing = true
    }

    private func deliver(_ urls: [URL]) {
        guard !urls.isEmpty else { return }
        if let target {
            engine.send(urls, to: target.identity.id)
        } else {
            staged = urls
        }
    }

    private func loadPhotos(_ picked: [PhotosPickerItem]) {
        guard !picked.isEmpty else { return }
        photos = []
        Task {
            var urls: [URL] = []
            for item in picked {
                if let media = try? await item.loadTransferable(type: PickedMedia.self) { urls.append(media.url) }
            }
            deliver(urls)
        }
    }
}
