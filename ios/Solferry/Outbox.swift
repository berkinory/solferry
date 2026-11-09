import CoreTransferable
import Darwin
import Foundation
import UniformTypeIdentifiers

/// Turns what the user picked into open files the core can read after the picker's access ends.
enum Outbox {
    private static let directory = URL.temporaryDirectory.appending(path: "Outbox", directoryHint: .isDirectory)

    /// The core takes ownership of the descriptor and closes it when the transfer ends.
    nonisolated static func open(_ url: URL) -> SharedFile? {
        let scoped = url.startAccessingSecurityScopedResource()
        defer { if scoped { url.stopAccessingSecurityScopedResource() } }
        var descriptor: Int32 = -1
        var failure: NSError?
        NSFileCoordinator().coordinate(readingItemAt: url, options: .withoutChanges, error: &failure) { readable in
            descriptor = Darwin.open(readable.path, O_RDONLY)
        }
        guard descriptor >= 0 else { return nil }
        return SharedFile(name: url.lastPathComponent, fd: descriptor)
    }

    /// Open descriptors keep the bytes of a temporary copy alive after its name is gone.
    nonisolated static func discard(_ urls: [URL]) {
        for url in urls where url.path.hasPrefix(directory.path) {
            try? FileManager.default.removeItem(at: url.deletingLastPathComponent())
        }
    }

    static func temporaryCopy(of source: URL) throws -> URL {
        let folder = directory.appending(path: UUID().uuidString, directoryHint: .isDirectory)
        try FileManager.default.createDirectory(at: folder, withIntermediateDirectories: true)
        let copy = folder.appending(path: source.lastPathComponent)
        try FileManager.default.copyItem(at: source, to: copy)
        return copy
    }
}

/// A photo or video from the picker, copied somewhere it can be read for as long as sending takes.
struct PickedMedia: Transferable {
    let url: URL

    static var transferRepresentation: some TransferRepresentation {
        FileRepresentation(importedContentType: .item) { received in
            PickedMedia(url: try Outbox.temporaryCopy(of: received.file))
        }
    }
}
