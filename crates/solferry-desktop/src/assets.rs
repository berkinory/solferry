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
    Alert => "alert",
    Android => "android",
    Apple => "apple",
    ArrowLeft => "arrow-left",
    Blocked => "blocked",
    Cancel => "cancel",
    Check => "check",
    Computer => "computer",
    Copy => "copy",
    Delete => "delete",
    Devices => "devices",
    Download => "download",
    External => "external",
    File => "file",
    Folder => "folder",
    FolderOpen => "folder-open",
    Key => "key",
    Linux => "linux",
    More => "more",
    Nearby => "nearby",
    Qr => "qr",
    Relay => "relay",
    Send => "send",
    Settings => "settings",
    Shield => "shield",
    Transfers => "transfers",
    Upload => "upload",
    View => "view",
    Windows => "windows",
}

pub fn icon(glyph: Glyph, size: f32) -> Svg {
    svg().path(glyph.path()).size(px(size)).flex_none()
}

pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if path == MARK {
            return Ok(Some(Cow::Borrowed(include_bytes!("../assets/mark.svg"))));
        }
        if let Some(bytes) = glyph_bytes(path) {
            return Ok(Some(Cow::Borrowed(bytes)));
        }
        gpui_kit::assets::Assets.load(path)
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        gpui_kit::assets::Assets.list(path)
    }
}
