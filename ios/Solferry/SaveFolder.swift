import Foundation

/// A folder the user picked in Files, which received files go to instead of the Solferry folder.
struct SaveFolder: Equatable {
    let url: URL
    let name: String

    private static let bookmarkKey = "saveFolderBookmark"
    private static let nameKey = "saveFolderName"

    /// What the picked folder was called, even after it stopped resolving. Nil when none was picked.
    static var pickedName: String? { UserDefaults.standard.string(forKey: nameKey) }

    /// Keeps a way back into a folder the picker returned, across launches. Nil if iOS won't give one.
    static func keep(_ picked: URL) -> SaveFolder? {
        let scoped = picked.startAccessingSecurityScopedResource()
        defer { if scoped { picked.stopAccessingSecurityScopedResource() } }
        guard let bookmark = try? picked.bookmarkData(options: .minimalBookmark, includingResourceValuesForKeys: nil, relativeTo: nil) else {
            return nil
        }
        let folder = SaveFolder(url: picked, name: picked.lastPathComponent)
        UserDefaults.standard.set(bookmark, forKey: bookmarkKey)
        UserDefaults.standard.set(folder.name, forKey: nameKey)
        return folder
    }

    static func forget() {
        UserDefaults.standard.removeObject(forKey: bookmarkKey)
        UserDefaults.standard.removeObject(forKey: nameKey)
    }

    /// The picked folder as it is now, or nil once the bookmark leads nowhere. A stale one is renewed while it still works.
    static func resolve() -> SaveFolder? {
        guard let bookmark = UserDefaults.standard.data(forKey: bookmarkKey) else { return nil }
        var stale = false
        guard let url = try? URL(resolvingBookmarkData: bookmark, bookmarkDataIsStale: &stale) else { return nil }
        let scoped = url.startAccessingSecurityScopedResource()
        defer { if scoped { url.stopAccessingSecurityScopedResource() } }
        guard (try? url.checkResourceIsReachable()) == true else { return nil }
        if stale {
            guard let renewed = try? url.bookmarkData(options: .minimalBookmark, includingResourceValuesForKeys: nil, relativeTo: nil) else {
                return nil
            }
            UserDefaults.standard.set(renewed, forKey: bookmarkKey)
        }
        UserDefaults.standard.set(url.lastPathComponent, forKey: nameKey)
        return SaveFolder(url: url, name: url.lastPathComponent)
    }
}
