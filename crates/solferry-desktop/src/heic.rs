//! Turns HEIC photos into JPEG on a Mac before they go to devices that can't open HEIC:
//! Linux, and Windows without its Store extension. Other systems can't decode HEIC themselves.

use std::path::{Path, PathBuf};

use solferry_core::Platform;
use tempfile::TempDir;

pub const fn wanted(platform: Option<Platform>) -> bool {
    matches!(platform, Some(Platform::Linux | Platform::Windows))
}

/// The paths to send, with HEIC photos swapped for JPEG copies that live as long as this does.
pub struct Staged {
    _folder: TempDir,
    pub paths: Vec<PathBuf>,
}

/// Converts the HEIC files among `paths`, or returns `None` when there are none.
/// A photo that fails to convert goes as it is.
pub fn stage(paths: &[PathBuf]) -> Option<Staged> {
    if !paths.iter().any(|path| is_heic(path)) {
        return None;
    }
    let folder = tempfile::Builder::new()
        .prefix("solferry-heic-")
        .tempdir()
        .inspect_err(|error| tracing::warn!(%error, "sending HEIC as is, no staging folder"))
        .ok()?;
    let paths = paths
        .iter()
        .map(|path| {
            if !is_heic(path) {
                return path.clone();
            }
            let target = folder.path().join(jpeg_name(path));
            match convert(path, &target) {
                Ok(()) => target,
                Err(error) => {
                    tracing::warn!(path = %path.display(), error, "sending HEIC as is");
                    path.clone()
                }
            }
        })
        .collect();
    Some(Staged { _folder: folder, paths })
}

fn is_heic(path: &Path) -> bool {
    path.is_file()
        && path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("heic") || ext.eq_ignore_ascii_case("heif"))
}

fn jpeg_name(path: &Path) -> PathBuf {
    let mut name = PathBuf::from(path.file_stem().unwrap_or_default());
    name.set_extension("jpg");
    name
}

/// Re-encodes the first image at 0.95 with ImageIO, which carries over its metadata,
/// orientation, dates and GPS included, without adding any.
#[cfg(target_os = "macos")]
fn convert(source: &Path, target: &Path) -> Result<(), &'static str> {
    use objc2_core_foundation::{CFDictionary, CFNumber, CFString, CFURL};
    use objc2_image_io::{
        CGImageDestination, CGImageSource, kCGImageDestinationLossyCompressionQuality,
    };

    let from = CFURL::from_file_path(source).ok_or("unusable path")?;
    let to = CFURL::from_file_path(target).ok_or("unusable path")?;
    // SAFETY: plain ImageIO calls on valid CF objects, with no options dictionaries but the
    // CFString to CFNumber one ImageIO documents for the quality.
    unsafe {
        let image = CGImageSource::with_url(&from, None).ok_or("unreadable")?;
        let kind = image.r#type().ok_or("not an image")?.to_string();
        if kind != "public.heic" && kind != "public.heif" {
            return Err("ImageIO doesn't read it as HEIC");
        }
        let jpeg = CFString::from_static_str("public.jpeg");
        let destination = CGImageDestination::with_url(&to, &jpeg, 1, None).ok_or("unwritable")?;
        let quality = CFNumber::new_f64(0.95);
        let options =
            CFDictionary::from_slices(&[kCGImageDestinationLossyCompressionQuality], &[&*quality]);
        destination.add_image_from_source(&image, 0, Some(options.as_opaque()));
        if destination.finalize() { Ok(()) } else { Err("encoding failed") }
    }
}

#[cfg(not(target_os = "macos"))]
fn convert(_: &Path, _: &Path) -> Result<(), &'static str> {
    Err("only macOS decodes HEIC")
}
