import Foundation

/// History knows where the core put a transfer's first file, but `Exporter` moves it on. This keeps a bookmark to
/// where it went, so tapping the transfer opens it, even inside a picked folder. The core reuses a path once its file
/// has moved out, so a record also notes when it was made, and a transfer takes the first one made after it finished.
enum SavedFiles {
    private struct Record: Codable {
        let path: String
        let at: UInt64
        let bookmark: Data
    }

    private static let kept = 200
    private static let lock = NSLock()
    nonisolated(unsafe) private static var expected = Set<String>()
    private static let file: URL = {
        let directory = URL.applicationSupportDirectory
        try? FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        return directory.appending(path: "saved-files.json")
    }()

    /// A transfer finished with its first file at `path`. The next export that moves it writes down where.
    nonisolated static func expect(_ path: String) {
        lock.withLock { _ = expected.insert(path) }
    }

    /// Called while the picked folder is open to this app, which a bookmark needs to reach it later.
    nonisolated static func moved(_ path: String, to target: URL) {
        lock.withLock {
            guard expected.remove(path) != nil,
                  let bookmark = try? target.bookmarkData(options: .minimalBookmark, includingResourceValuesForKeys: nil, relativeTo: nil)
            else { return }
            let record = Record(path: path, at: now(), bookmark: bookmark)
            let records = [record] + load().prefix(kept - 1)
            try? JSONEncoder().encode(records).write(to: file, options: .atomic)
        }
    }

    /// Where the first file of a transfer that finished at `finishedMs` is now. Nil once it's gone.
    /// The caller starts and stops access to it, since a file in a picked folder is only readable in between.
    static func find(_ path: String, finishedMs: UInt64) -> URL? {
        let record = lock.withLock { load().filter { $0.path == path && $0.at >= finishedMs }.min { $0.at < $1.at } }
        guard let record else { return nil }
        var stale = false
        guard let url = try? URL(resolvingBookmarkData: record.bookmark, bookmarkDataIsStale: &stale) else { return nil }
        let scoped = url.startAccessingSecurityScopedResource()
        defer { if scoped { url.stopAccessingSecurityScopedResource() } }
        return (try? url.checkResourceIsReachable()) == true ? url : nil
    }

    private nonisolated static func load() -> [Record] {
        guard let data = try? Data(contentsOf: file) else { return [] }
        return (try? JSONDecoder().decode([Record].self, from: data)) ?? []
    }

    private nonisolated static func now() -> UInt64 {
        UInt64(Date().timeIntervalSince1970 * 1000)
    }
}
