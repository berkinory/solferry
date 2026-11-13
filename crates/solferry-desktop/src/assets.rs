use std::borrow::Cow;

use gpui_kit::{AssetSource, Result, SharedString, Styled, Svg, px, svg};

pub const MARK: &str = "brand/mark.svg";

macro_rules! glyphs {
    ($($variant:ident => $file:literal),* $(,)?) => {
        /// The icons the app draws, from assets/icons.
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum Glyph {
            $($variant),*
        }

        impl Glyph {
            pub const fn path(self) -> &'static str {
                match self {
                    $(Self::$variant => concat!("icons/app/", $file, ".svg")),*
                }
            }
        }

        fn glyph_bytes(path: &str) -> Option<&'static [u8]> {
            match path {
                $(concat!("icons/app/", $file, ".svg") => {
                    Some(include_bytes!(concat!("../assets/icons/", $file, ".svg")))
                })*
                _ => None,
            }
        }
    };
}

glyphs! {
    Add => "add",
    Android => "android",
    Apple => "apple",
    Appearance => "appearance",
    Blocked => "blocked",
    Cancel => "cancel",
    Check => "check",
    Clipboard => "clipboard",
    Computer => "computer",
    Copy => "copy",
    Cursor => "cursor",
    Delete => "delete",
    Devices => "devices",
    Download => "download",
    External => "external",
    FanCard => "fan-card",
    File => "file",
    Folder => "folder",
    FolderOpen => "folder-open",
    History => "history",
    Home => "home",
    Key => "key",
    Language => "language",
    Link => "link",
    Linux => "linux",
    ChevronDown => "chevron-down",
    ChevronUp => "chevron-up",
    Nearby => "nearby",
    Notification => "notification",
    Qr => "qr",
    Relay => "relay",
    Send => "send",
    Settings => "settings",
    Shield => "shield",
    TypeArchive => "type-archive",
    TypeAudio => "type-audio",
    TypeCode => "type-code",
    TypeDoc => "type-doc",
    TypeImage => "type-image",
    TypePackage => "type-package",
    TypePdf => "type-pdf",
    TypeSheet => "type-sheet",
    TypeSlides => "type-slides",
    TypeText => "type-text",
    TypeVideo => "type-video",
    Upload => "upload",
    View => "view",
    Windows => "windows",
}

/// The icon for a file, from what its name says it is.
pub fn file_glyph(name: &str) -> Glyph {
    let extension = name.rsplit_once('.').map_or("", |(_, extension)| extension).to_lowercase();
    match extension.as_str() {
        "jpg" | "jpeg" | "png" | "gif" | "webp" | "heic" | "heif" | "bmp" | "tiff" | "svg"
        | "raw" => Glyph::TypeImage,
        "mp4" | "mov" | "mkv" | "avi" | "webm" | "m4v" => Glyph::TypeVideo,
        "mp3" | "wav" | "flac" | "aac" | "ogg" | "m4a" | "opus" => Glyph::TypeAudio,
        "pdf" => Glyph::TypePdf,
        "doc" | "docx" | "odt" | "rtf" | "pages" => Glyph::TypeDoc,
        "xls" | "xlsx" | "ods" | "csv" | "numbers" => Glyph::TypeSheet,
        "ppt" | "pptx" | "odp" | "key" => Glyph::TypeSlides,
        "zip" | "rar" | "7z" | "tar" | "gz" | "tgz" | "xz" | "bz2" => Glyph::TypeArchive,
        "txt" | "md" | "log" => Glyph::TypeText,
        "rs" | "js" | "ts" | "tsx" | "py" | "go" | "c" | "cpp" | "h" | "java" | "kt" | "swift"
        | "html" | "css" | "json" | "toml" | "yaml" | "yml" | "sh" => Glyph::TypeCode,
        "apk" | "exe" | "msi" | "dmg" | "pkg" | "deb" | "rpm" | "appimage" | "ipa" => {
            Glyph::TypePackage
        }
        _ => Glyph::File,
    }
}

pub fn icon(glyph: Glyph, size: f32) -> Svg {
    svg().path(glyph.path()).size(px(size)).flex_none()
}

/// Flags for the language picker, drawn in their own colors.
fn flag_bytes(path: &str) -> Option<&'static [u8]> {
    Some(match path {
        "flags/gb.svg" => include_bytes!("../assets/flags/gb.svg"),
        "flags/tr.svg" => include_bytes!("../assets/flags/tr.svg"),
        "flags/de.svg" => include_bytes!("../assets/flags/de.svg"),
        "flags/fr.svg" => include_bytes!("../assets/flags/fr.svg"),
        "flags/it.svg" => include_bytes!("../assets/flags/it.svg"),
        "flags/es.svg" => include_bytes!("../assets/flags/es.svg"),
        "flags/br.svg" => include_bytes!("../assets/flags/br.svg"),
        "flags/ru.svg" => include_bytes!("../assets/flags/ru.svg"),
        "flags/ua.svg" => include_bytes!("../assets/flags/ua.svg"),
        "flags/jp.svg" => include_bytes!("../assets/flags/jp.svg"),
        "flags/kr.svg" => include_bytes!("../assets/flags/kr.svg"),
        "flags/cn.svg" => include_bytes!("../assets/flags/cn.svg"),
        "flags/nl.svg" => include_bytes!("../assets/flags/nl.svg"),
        "flags/pl.svg" => include_bytes!("../assets/flags/pl.svg"),
        _ => return None,
    })
}

pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if path == MARK {
            return Ok(Some(Cow::Borrowed(include_bytes!("../assets/mark.svg"))));
        }
        if let Some(bytes) = glyph_bytes(path).or_else(|| flag_bytes(path)) {
            return Ok(Some(Cow::Borrowed(bytes)));
        }
        gpui_kit::assets::Assets.load(path)
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        gpui_kit::assets::Assets.list(path)
    }
}
