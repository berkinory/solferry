import UIKit
import UniformTypeIdentifiers

/// The app group both the app and this extension are signed into.
private let appGroup = "group.app.solferry"
/// Copies the app never picked up get cleared after this long.
private let keepCopiesFor: TimeInterval = 24 * 60 * 60

/// Copies what was shared into the app group and opens Solferry on its send sheet. The extension
/// never starts the core: it would not fit in an extension's memory limit.
@objc(ShareViewController)
final class ShareViewController: UIViewController {
    private var started = false

    override func viewDidLoad() {
        super.viewDidLoad()
        let spinner = UIActivityIndicatorView(style: .large)
        spinner.translatesAutoresizingMaskIntoConstraints = false
        spinner.startAnimating()
        view.addSubview(spinner)
        NSLayoutConstraint.activate([
            spinner.centerXAnchor.constraint(equalTo: view.centerXAnchor),
            spinner.centerYAnchor.constraint(equalTo: view.centerYAnchor),
        ])
    }

    override func viewDidAppear(_ animated: Bool) {
        super.viewDidAppear(animated)
        guard !started else { return }
        started = true
        guard let shared = FileManager.default
            .containerURL(forSecurityApplicationGroupIdentifier: appGroup)?
            .appending(path: "Shared", directoryHint: .isDirectory)
        else { return fail() }
        clearStale(shared)
        let batch = UUID().uuidString
        let items = extensionContext?.inputItems as? [NSExtensionItem] ?? []
        collect(items.flatMap { $0.attachments ?? [] }, into: shared.appending(path: batch)) { [weak self] copied in
            guard let self else { return }
            guard copied > 0, let url = URL(string: "solferry://send?batch=\(batch)") else { return fail() }
            openApp(url)
        }
    }

    /// Extensions can't reach `UIApplication.shared`, but the application sits at the end of the
    /// responder chain. From iOS 18 the old `openURL:` only logs and returns false, so the
    /// modern call is used there.
    private func openApp(_ url: URL) {
        var responder: UIResponder? = self
        while let current = responder {
            if let application = current as? UIApplication {
                if #available(iOS 18.0, *) {
                    application.open(url, options: [:]) { _ in self.finish() }
                } else {
                    application.perform(Selector(("openURL:")), with: url)
                    finish()
                }
                return
            }
            responder = current.next
        }
        fail()
    }

    private func finish() {
        extensionContext?.completeRequest(returningItems: nil)
    }

    private func fail() {
        extensionContext?.cancelRequest(withError: CocoaError(.fileReadUnknown))
    }
}

/// Copies every shared item into `batch/<position>/`, and counts the ones that made it.
private func collect(_ providers: [NSItemProvider], into batch: URL, done: @escaping (Int) -> Void) {
    var copied = 0
    let lock = NSLock()
    let group = DispatchGroup()
    for (index, provider) in providers.enumerated() {
        let type = provider.registeredTypeIdentifiers.first { UTType($0)?.conforms(to: .data) == true }
            ?? provider.registeredTypeIdentifiers.first { UTType($0)?.conforms(to: .directory) == true }
        guard let type else { continue }
        group.enter()
        // The file only exists until this handler returns.
        provider.loadFileRepresentation(forTypeIdentifier: type) { file, _ in
            if let file, (try? copy(file, into: batch.appending(path: String(index)))) != nil {
                lock.lock()
                copied += 1
                lock.unlock()
            }
            group.leave()
        }
    }
    group.notify(queue: .main) { done(copied) }
}

private func copy(_ file: URL, into folder: URL) throws {
    try FileManager.default.createDirectory(at: folder, withIntermediateDirectories: true)
    try FileManager.default.copyItem(at: file, to: folder.appending(path: file.lastPathComponent))
}

private func clearStale(_ shared: URL) {
    let files = FileManager.default
    let batches = (try? files.contentsOfDirectory(at: shared, includingPropertiesForKeys: [.creationDateKey])) ?? []
    for batch in batches {
        let created = try? batch.resourceValues(forKeys: [.creationDateKey]).creationDate
        if let created, -created.timeIntervalSinceNow > keepCopiesFor { try? files.removeItem(at: batch) }
    }
}
