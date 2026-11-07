use std::path::{Path, PathBuf};

const RESERVED: [&str; 22] = [
    "con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8",
    "com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
];
const MAX_PATH_LEN: usize = 4096;
const MAX_LABEL_CHARS: usize = 64;

/// Converts a peer-supplied path into a relative path that cannot escape the destination.
pub(crate) fn sanitize(path: &str) -> Option<PathBuf> {
    if path.is_empty() || path.len() > MAX_PATH_LEN {
        return None;
    }
    path.split('/').all(is_safe_component).then(|| path.split('/').collect())
}

fn is_safe_component(component: &str) -> bool {
    let has_forbidden_char = component
        .chars()
        .any(|c| is_deceptive(c) || matches!(c, '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|'));
    let stem = component.split('.').next().unwrap_or(component).to_ascii_lowercase();
    !component.is_empty()
        && !has_forbidden_char
        && !component.ends_with(['.', ' '])
        && !RESERVED.contains(&stem.as_str())
}

fn is_deceptive(c: char) -> bool {
    c.is_control()
        || matches!(c, '\u{200e}' | '\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
}

/// Makes a peer-chosen display name safe to show: no terminal escapes, no direction tricks.
pub(crate) fn clean_label(label: &str) -> String {
    let clean: String = label.chars().filter(|c| !is_deceptive(*c)).take(MAX_LABEL_CHARS).collect();
    if clean.trim().is_empty() { "Unnamed device".to_owned() } else { clean }
}

/// True when a path inside `destination` already passes through a symbolic link.
pub(crate) fn crosses_symlink(destination: &Path, relative: &Path) -> bool {
    let mut current = destination.to_owned();
    relative.components().any(|component| {
        current.push(component);
        current.symlink_metadata().is_ok_and(|metadata| metadata.file_type().is_symlink())
    })
}

#[cfg(test)]
mod tests {
    use super::{clean_label, sanitize};

    #[test]
    fn keeps_ordinary_nested_paths() {
        assert!(sanitize("Fotoğraflar/2026/tatil .final.jpg").is_some());
    }

    #[test]
    fn rejects_paths_that_escape_or_alias_the_destination() {
        let attacks = [
            "",
            "/etc/passwd",
            "../secret",
            "a/../../b",
            "a//b",
            "a/./b",
            "C:\\Windows\\win.ini",
            "C:/Windows",
            "..\\..\\x",
            "a\\b",
            "ok/nul.txt",
            "CON",
            "x\0y",
            "trail.",
            "trail ",
            "stream:ads",
            "invoice\u{202e}fdp.exe",
        ];
        for attack in attacks {
            assert!(sanitize(attack).is_none(), "{attack:?} must be rejected");
        }
    }

    #[test]
    fn strips_terminal_escapes_from_labels() {
        assert_eq!(clean_label("evil\u{1b}[2Jname"), "evil[2Jname");
        assert_eq!(clean_label("\n\t"), "Unnamed device");
    }
}
