use std::path::PathBuf;

const RESERVED: [&str; 22] = [
    "con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8",
    "com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
];
const MAX_PATH_LEN: usize = 4096;

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
        .any(|c| c.is_control() || matches!(c, '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|'));
    let stem = component.split('.').next().unwrap_or(component).to_ascii_lowercase();
    !component.is_empty()
        && !has_forbidden_char
        && !component.ends_with(['.', ' '])
        && !RESERVED.contains(&stem.as_str())
}

#[cfg(test)]
mod tests {
    use super::sanitize;

    #[test]
    fn keeps_ordinary_nested_paths() {
        assert!(sanitize("Fotoğraflar/2026/tatil .final.jpg").is_some());
    }

    #[test]
    fn rejects_paths_that_escape_or_alias_the_destination() {
        let attacks = [
            "", "/etc/passwd", "../secret", "a/../../b", "a//b", "a/./b", "C:\\Windows\\win.ini",
            "C:/Windows", "..\\..\\x", "a\\b", "ok/nul.txt", "CON", "x\0y", "trail.", "trail ",
            "stream:ads",
        ];
        for attack in attacks {
            assert!(sanitize(attack).is_none(), "{attack:?} must be rejected");
        }
    }
}
