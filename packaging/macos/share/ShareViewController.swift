import AppKit
import UniformTypeIdentifiers

/// The app group both the app and this extension are signed into.
private let appGroup = "Z66C58Z3RC.app.solferry"
/// Copies the app never picked up, because a send never started, get cleared after this long.
private let keepCopiesFor: TimeInterval = 24 * 60 * 60

/// Hands what was shared to Solferry, which opens its send sheet with it. Files from Finder go
/// over as they are. Photos and other apps share temporary copies that live inside this
/// extension's sandbox, so those are copied into the app group first, where the app can read them.
@objc(ShareViewController)
final class ShareViewController: NSViewController {
    override func loadView() {
        view = NSView(frame: .zero)
    }

    override func viewDidLoad() {
        super.viewDidLoad()
        let items = extensionContext?.inputItems as? [NSExtensionItem] ?? []
        let providers = items.flatMap { $0.attachments ?? [] }
        let batch = FileManager.default
            .containerURL(forSecurityApplicationGroupIdentifier: appGroup)?
            .appendingPathComponent("Shared", isDirectory: true)
        if let batch { clearStale(batch) }
        collect(providers, into: batch?.appendingPathComponent(UUID().uuidString, isDirectory: true)) { [weak self] urls in
            self?.open(urls)
        }
    }

    private func open(_ urls: [URL]) {
        guard !urls.isEmpty else { return fail() }
        // The extension sits in Solferry.app/Contents/PlugIns, so this opens the very same app.
        let app = Bundle.main.bundleURL.deletingLastPathComponent().deletingLastPathComponent()
            .deletingLastPathComponent()
        NSWorkspace.shared.open(urls, withApplicationAt: app, configuration: NSWorkspace.OpenConfiguration()) { _, error in
            DispatchQueue.main.async {
                if let error { return self.fail(error) }
                self.extensionContext?.completeRequest(returningItems: nil)
            }
        }
    }

    private func fail(_ error: Error = CocoaError(.fileReadUnknown)) {
        extensionContext?.cancelRequest(withError: error)
    }
}

/// Loads every shared item as a file, keeping the order they were shared in.
private func collect(_ providers: [NSItemProvider], into batch: URL?, done: @escaping ([URL]) -> Void) {
    var urls = [URL?](repeating: nil, count: providers.count)
    let lock = NSLock()
    let group = DispatchGroup()
    for (index, provider) in providers.enumerated() {
        group.enter()
        load(provider, into: batch?.appendingPathComponent(String(index), isDirectory: true)) { url in
            lock.lock()
            urls[index] = url
            lock.unlock()
            group.leave()
        }
    }
    group.notify(queue: .main) { done(urls.compactMap { $0 }) }
}

private func load(_ provider: NSItemProvider, into folder: URL?, done: @escaping (URL?) -> Void) {
    let fileURL = UTType.fileURL.identifier
    if provider.hasItemConformingToTypeIdentifier(fileURL) {
        provider.loadItem(forTypeIdentifier: fileURL) { item, _ in
            done((item as? URL) ?? (item as? Data).flatMap { URL(dataRepresentation: $0, relativeTo: nil) })
        }
        return
    }
    let type = provider.registeredTypeIdentifiers.first { UTType($0)?.conforms(to: .data) == true }
    guard let type, let folder else { return done(nil) }
    // The file only exists until this handler returns.
    provider.loadFileRepresentation(forTypeIdentifier: type) { file, _ in
        done(file.flatMap { try? copy($0, into: folder) })
    }
}

private func copy(_ file: URL, into folder: URL) throws -> URL {
    try FileManager.default.createDirectory(at: folder, withIntermediateDirectories: true)
    let copy = folder.appendingPathComponent(file.lastPathComponent)
    try FileManager.default.copyItem(at: file, to: copy)
    return copy
}

private func clearStale(_ shared: URL) {
    let files = FileManager.default
    let batches = (try? files.contentsOfDirectory(at: shared, includingPropertiesForKeys: [.creationDateKey])) ?? []
    for batch in batches {
        let created = try? batch.resourceValues(forKeys: [.creationDateKey]).creationDate
        if let created, -created.timeIntervalSinceNow > keepCopiesFor { try? files.removeItem(at: batch) }
    }
}
