import SwiftUI

enum Source {
    case photos, files
}

/// Where the files come from: the photo picker or the file browser.
struct SourceScreen: View {
    @Environment(\.dismiss) private var dismiss
    let choose: (Source) -> Void

    var body: some View {
        SheetPage(title: "", close: { dismiss() }) {
            HStack(spacing: SpaceTokens.sm) {
                SourceTile(label: "Photos and videos", glyph: .typeImage) { pick(.photos) }
                SourceTile(label: "Files", glyph: .folder) { pick(.files) }
            }
        }
    }

    private func pick(_ source: Source) {
        choose(source)
        dismiss()
    }
}

private struct SourceTile: View {
    @Environment(\.palette) private var palette
    let label: LocalizedStringKey
    let glyph: Glyph
    let action: () -> Void

    var body: some View {
        CardFrame {
            Well(action: action) {
                VStack(spacing: SpaceTokens.sm) {
                    Icon(glyph, tint: palette.ink, size: 32)
                    Text(label)
                        .font(.subheadline.weight(.semibold))
                        .multilineTextAlignment(.center)
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
                .aspectRatio(1, contentMode: .fit)
            }
        }
    }
}
