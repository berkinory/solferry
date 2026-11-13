import Darwin
import Foundation

/// Private copies of files queued for a device that isn't there, kept until the send is over.
/// Picked files are only readable while the picker's access lasts, so a copy is what stays.
enum QueueFiles {
    private static let directory = URL.applicationSupportDirectory.appending(path: "Queue", directoryHint: .isDirectory)

    /// The reference the core keeps for the copy, which is its path.
    nonisolated static func keep(_ source: URL) -> QueuedFile? {
        let scoped = source.startAccessingSecurityScopedResource()
        defer { if scoped { source.stopAccessingSecurityScopedResource() } }
        let folder = directory.appending(path: UUID().uuidString, directoryHint: .isDirectory)
        let copy = folder.appending(path: source.lastPathComponent)
        var failure: NSError?
        var copied = false
        NSFileCoordinator().coordinate(readingItemAt: source, options: .withoutChanges, error: &failure) { readable in
            copied = (try? FileManager.default.createDirectory(at: folder, withIntermediateDirectories: true)) != nil
                && (try? FileManager.default.copyItem(at: readable, to: copy)) != nil
        }
        guard copied else {
            try? FileManager.default.removeItem(at: folder)
            return nil
        }
        return QueuedFile(name: source.lastPathComponent, reference: copy.path)
    }

    /// The core takes ownership of the descriptor and closes it when the transfer ends.
    nonisolated static func reopen(_ reference: String) -> Int32? {
        let descriptor = Darwin.open(reference, O_RDONLY)
        return descriptor >= 0 ? descriptor : nil
    }

    nonisolated static func release(_ reference: String) {
        let folder = URL(filePath: reference).deletingLastPathComponent()
        guard folder.deletingLastPathComponent().standardizedFileURL == directory.standardizedFileURL else { return }
        try? FileManager.default.removeItem(at: folder)
    }
}
