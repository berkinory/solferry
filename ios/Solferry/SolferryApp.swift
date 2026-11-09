import PhotosUI
import SwiftUI

private let presenceEverySeconds = 20.0

private enum Presented: Identifiable {
    case pairing, code, send

    var id: Self { self }
}

enum Place: Hashable {
    case home, history, settings
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
    @State private var place = Place.home
    @State private var pairing = false
    @State private var sending = false
    @State private var staged: [URL] = []
    @State private var target: PairedDevice?
    @State private var choosing = false
    @State private var browsing = false
    @State private var browsingPhotos = false
    @State private var photos: [PhotosPickerItem] = []

    var body: some View {
        TabView(selection: $place) {
            NavigationStack { HomeScreen(pick: pick) }
            .tabItem { Label { Text("Transfer") } icon: { Image(decorative: Glyph.home.rawValue).renderingMode(.template) } }
            .tag(Place.home)
            NavigationStack { HistoryScreen() }
                .tabItem { Label { Text("History") } icon: { Image(decorative: Glyph.history.rawValue).renderingMode(.template) } }
                .tag(Place.history)
            NavigationStack { SettingsScreen() }
                .tabItem { Label { Text("Settings") } icon: { Image(decorative: Glyph.settings.rawValue).renderingMode(.template) } }
                .tag(Place.settings)
        }
        .environment(\.addDevice) { pairing = true }
        .sensoryFeedback(.selection, trigger: place)
        .overlay(alignment: .top) { Toaster() }
        .sheet(item: presented) { sheet in
            switch sheet {
            case .pairing: PairingScreen()
            case .code: AddDeviceScreen()
            case .send: SendScreen(files: $staged)
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
        .onChange(of: staged) { _, files in if !files.isEmpty { sending = true } }
        .onChange(of: engine.state.pairedCount) { goHome() }
        .onChange(of: engine.state.asking) { before, now in if now > before { goHome() } }
        .onChange(of: phase, initial: true) { _, now in engine.sceneChanged(active: now == .active) }
        .task(id: phase) {
            while phase == .active, !Task.isCancelled {
                engine.bridge.checkPresence()
                try? await Task.sleep(for: .seconds(presenceEverySeconds))
            }
        }
    }

    /// One sheet at a time, and a device asking to pair goes ahead of the others.
    private var presented: Binding<Presented?> {
        Binding {
            let state = engine.state
            if !state.pairings.isEmpty || state.pairingCode != nil { return .pairing }
            if pairing { return .code }
            return sending ? .send : nil
        } set: { value in
            guard value == nil else { return }
            if !engine.state.pairings.isEmpty || engine.state.pairingCode != nil {
                engine.closePairing()
            } else if pairing {
                pairing = false
            } else {
                sending = false
                Outbox.discard(staged)
                staged = []
            }
        }
    }

    private func goHome() {
        place = .home
        pairing = false
    }

    private func pick(_ device: PairedDevice?) {
        target = device
        choosing = true
    }

    private func deliver(_ urls: [URL]) {
        guard !urls.isEmpty else { return }
        if let target {
            engine.send(urls, to: [target.identity.id])
        } else {
            staged += urls
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
