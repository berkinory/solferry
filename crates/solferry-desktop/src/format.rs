use std::time::{SystemTime, UNIX_EPOCH};

const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];

pub fn bytes(value: u64) -> String {
    let mut amount = precise(value);
    let mut unit = 0;
    while amount >= 1024.0 && unit < UNITS.len() - 1 {
        amount /= 1024.0;
        unit += 1;
    }
    if unit == 0 { format!("{value} B") } else { format!("{amount:.1} {}", UNITS[unit]) }
}

pub fn duration(seconds: f64) -> String {
    let rounded = seconds.round();
    if rounded < 60.0 {
        format!("{rounded:.0}s")
    } else if rounded < 3600.0 {
        format!("{:.0}m {:.0}s", (rounded / 60.0).floor(), rounded % 60.0)
    } else {
        format!("{:.0}h {:.0}m", (rounded / 3600.0).floor(), ((rounded % 3600.0) / 60.0).floor())
    }
}

/// How long ago a moment was, in the largest unit that still reads naturally.
pub fn ago(ms: u64) -> String {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis();
    let minutes = now.saturating_sub(u128::from(ms)) / 60_000;
    match minutes {
        0 => "just now".to_owned(),
        1..60 => format!("{minutes} min ago"),
        60..1440 => format!("{} h ago", minutes / 60),
        1440..2880 => "yesterday".to_owned(),
        2880..43200 => format!("{} days ago", minutes / 1440),
        _ => format!("{} months ago", (minutes / 43200).max(1)),
    }
}

#[expect(clippy::cast_precision_loss, reason = "display values do not need full precision")]
pub fn precise(value: u64) -> f64 {
    value as f64
}

#[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "non-negative rates")]
pub fn whole(value: f64) -> u64 {
    value.max(0.0) as u64
}
