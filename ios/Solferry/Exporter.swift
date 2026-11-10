import Darwin
import Foundation

/// The core receives into private storage. Finished files move into the Solferry folder in Files,
/// or the folder the user picked, keeping their folders.
enum Exporter {
    static let staging: URL = {
        var directory = URL.applicationSupportDirectory.appending(path: "Received", directoryHint: .isDirectory)
        try? FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        var values = URLResourceValues()
        values.isExcludedFromBackup = true
        try? directory.setResourceValues(values)
        return directory
    }()

    private static let partialDirectory = ".solferry-partial"
    /// A move into a picked folder can be a copy that a killed app leaves half-written, so its path is written down first.
    private static let unfinishedKey = "unfinishedCopy"
    private static let lock = NSLock()

    /// False if a file couldn't be moved. It stays in private storage for the next attempt.
    nonisolated static func export(to folder: SaveFolder?) -> Bool {
        lock.withLock { moveAll(to: folder) }
    }

    /// Finishes what a killed app left behind: drops its half-written copy and moves the files again.
    nonisolated static func recover(to folder: SaveFolder?) -> Bool {
        lock.withLock {
            if let path = UserDefaults.standard.string(forKey: unfinishedKey) {
                if let folder { access(folder) { try? FileManager.default.removeItem(at: folder.url.appending(path: path)) } }
                UserDefaults.standard.removeObject(forKey: unfinishedKey)
            }
            return moveAll(to: folder)
        }
    }

    private nonisolated static func moveAll(to folder: SaveFolder?) -> Bool {
        guard let folder else { return moveAll(in: staging, path: [], to: URL.documentsDirectory, picked: false) }
        return access(folder) { moveAll(in: staging, path: [], to: folder.url, picked: true) }
    }

    private nonisolated static func access<T>(_ folder: SaveFolder, _ work: () -> T) -> T {
        let scoped = folder.url.startAccessingSecurityScopedResource()
        defer { if scoped { folder.url.stopAccessingSecurityScopedResource() } }
        return work()
    }

    private nonisolated static func moveAll(in directory: URL, path: [String], to root: URL, picked: Bool) -> Bool {
        let items = (try? FileManager.default.contentsOfDirectory(at: directory, includingPropertiesForKeys: [.isDirectoryKey])) ?? []
        var moved = true
        for item in items where item.lastPathComponent != partialDirectory {
            if (try? item.resourceValues(forKeys: [.isDirectoryKey]).isDirectory) == true {
                moved = moveAll(in: item, path: path + [item.lastPathComponent], to: root, picked: picked) && moved
                rmdir(item.path)
            } else {
                moved = move(item, path: path, to: root, picked: picked) && moved
            }
        }
        return moved
    }

    private nonisolated static func move(_ file: URL, path: [String], to root: URL, picked: Bool) -> Bool {
        let parent = path.reduce(root) { $0.appending(path: $1, directoryHint: .isDirectory) }
        guard (try? FileManager.default.createDirectory(at: parent, withIntermediateDirectories: true)) != nil else { return false }
        let target = free(parent.appending(path: file.lastPathComponent))
        if picked { UserDefaults.standard.set((path + [target.lastPathComponent]).joined(separator: "/"), forKey: unfinishedKey) }
        defer { if picked { UserDefaults.standard.removeObject(forKey: unfinishedKey) } }
        var moved = false
        var failure: NSError?
        NSFileCoordinator().coordinate(writingItemAt: target, options: .forReplacing, error: &failure) { writable in
            moved = (try? FileManager.default.moveItem(at: file, to: writable)) != nil
        }
        if !moved, picked { try? FileManager.default.removeItem(at: target) }
        return moved
    }

    /// The core names arrivals this way when a name is taken, so the folder reads the same either way.
    private nonisolated static func free(_ target: URL) -> URL {
        guard FileManager.default.fileExists(atPath: target.path) else { return target }
        let stem = target.deletingPathExtension().lastPathComponent
        let suffix = target.pathExtension.isEmpty ? "" : ".\(target.pathExtension)"
        let parent = target.deletingLastPathComponent()
        return (1...).lazy
            .map { parent.appending(path: "\(stem) (\($0))\(suffix)") }
            .first { !FileManager.default.fileExists(atPath: $0.path) } ?? target
    }
}
