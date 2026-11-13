import Foundation

private let detector = try? NSDataDetector(types: NSTextCheckingResult.CheckingType.link.rawValue)
private let schemes: Set<String> = ["http", "https", "mailto"]

/// Received text with its links underlined. They open only when tapped. A link counts only when
/// the sender wrote out its http, https or mailto scheme, never as a bare domain or address.
func linked(_ text: String) -> AttributedString {
    var result = AttributedString(text)
    guard let detector else { return result }
    for match in detector.matches(in: text, range: NSRange(text.startIndex..., in: text)) {
        guard let url = match.url, let scheme = url.scheme?.lowercased(), schemes.contains(scheme),
              let found = Range(match.range, in: text), text[found].lowercased().hasPrefix(scheme + ":"),
              let range = Range(found, in: result) else { continue }
        result[range].link = url
        result[range].underlineStyle = .single
    }
    return result
}
