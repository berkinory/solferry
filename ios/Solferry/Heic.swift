import Foundation
import ImageIO
import OSLog
import UniformTypeIdentifiers

/// Turns HEIC photos into JPEG for devices that can't open HEIC: Linux, and Windows without its Store extension.
enum Heic {
    static let key = "convertHeic"
    private nonisolated static let quality = 0.95
    private nonisolated static let log = Logger(subsystem: "app.solferry", category: "heic")

    static var enabled: Bool { UserDefaults.standard.object(forKey: key) as? Bool ?? true }

    static func wanted(_ platform: DevicePlatform?) -> Bool { platform == .linux || platform == .windows }

    /// JPEG copies of the HEIC photos among `urls`, kept in the outbox so `Outbox.discard` removes them.
    nonisolated static func jpegCopies(of urls: [URL]) -> [URL: URL] {
        var copies: [URL: URL] = [:]
        for url in urls where ["heic", "heif"].contains(url.pathExtension.lowercased()) {
            autoreleasepool { copies[url] = jpegCopy(of: url) }
        }
        return copies
    }

    /// Re-encodes the first image with all its metadata, orientation, dates and GPS included.
    /// Nil means the original goes as it is.
    private nonisolated static func jpegCopy(of url: URL) -> URL? {
        let scoped = url.startAccessingSecurityScopedResource()
        defer { if scoped { url.stopAccessingSecurityScopedResource() } }
        guard let source = CGImageSourceCreateWithURL(url as CFURL, nil),
              let type = CGImageSourceGetType(source).flatMap({ UTType($0 as String) }),
              type.conforms(to: .heic) || type.conforms(to: .heif)
        else {
            log.notice("sending \(url.lastPathComponent, privacy: .public) as is, ImageIO doesn't read it as HEIC")
            return nil
        }
        do {
            let target = try Outbox.freshFolder().appending(path: url.deletingPathExtension().lastPathComponent + ".jpg")
            guard let destination = CGImageDestinationCreateWithURL(target as CFURL, UTType.jpeg.identifier as CFString, 1, nil) else {
                throw CocoaError(.fileWriteUnknown)
            }
            let options = [kCGImageDestinationLossyCompressionQuality: quality] as CFDictionary
            CGImageDestinationAddImageFromSource(destination, source, 0, options)
            guard CGImageDestinationFinalize(destination) else {
                Outbox.discard([target])
                throw CocoaError(.fileWriteUnknown)
            }
            return target
        } catch {
            log.error("sending \(url.lastPathComponent, privacy: .public) as HEIC, conversion failed: \(error, privacy: .public)")
            return nil
        }
    }
}
