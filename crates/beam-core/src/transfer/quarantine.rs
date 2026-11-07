use std::path::Path;

/// Marks a received file as downloaded, so Gatekeeper and SmartScreen warn before it first runs.
#[cfg(target_os = "macos")]
pub(super) fn mark_downloaded(path: &Path) {
    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs());
    let value = format!("0081;{seconds:x};beam;");
    let _ = xattr::set(path, "com.apple.quarantine", value.as_bytes());
}

#[cfg(windows)]
pub(super) fn mark_downloaded(path: &Path) {
    let mut stream = path.as_os_str().to_owned();
    stream.push(":Zone.Identifier");
    let _ = std::fs::write(stream, "[ZoneTransfer]\r\nZoneId=3\r\n");
}

#[cfg(not(any(target_os = "macos", windows)))]
pub(super) fn mark_downloaded(_path: &Path) {}
