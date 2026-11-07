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

#[expect(clippy::cast_precision_loss, reason = "display values do not need full precision")]
pub fn precise(value: u64) -> f64 {
    value as f64
}

#[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "non-negative rates")]
pub fn whole(value: f64) -> u64 {
    value.max(0.0) as u64
}
