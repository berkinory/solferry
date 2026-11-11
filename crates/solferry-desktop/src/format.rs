use std::time::{SystemTime, UNIX_EPOCH};

use rust_i18n::t;

const LARGEST_UNIT: usize = 4;

pub fn bytes(value: u64) -> String {
    let mut amount = precise(value);
    let mut unit = 0;
    while amount >= 1024.0 && unit < LARGEST_UNIT {
        amount /= 1024.0;
        unit += 1;
    }
    let amount = format!("{amount:.1}");
    match unit {
        0 => t!("format.size.b", amount = value),
        1 => t!("format.size.kb", amount = amount),
        2 => t!("format.size.mb", amount = amount),
        3 => t!("format.size.gb", amount = amount),
        _ => t!("format.size.tb", amount = amount),
    }
    .into_owned()
}

/// How long ago a moment was, in the largest unit that still reads naturally.
pub fn ago(ms: u64) -> String {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis();
    let minutes = now.saturating_sub(u128::from(ms)) / 60_000;
    match minutes {
        0 => t!("format.ago.now"),
        1..60 => t!("format.ago.minutes", count = minutes),
        60..1440 => t!("format.ago.hours", count = minutes / 60),
        1440..2880 => t!("format.ago.yesterday"),
        2880..43200 => t!("format.ago.days", count = minutes / 1440),
        _ => t!("format.ago.months", count = (minutes / 43200).max(1)),
    }
    .into_owned()
}

#[expect(clippy::cast_precision_loss, reason = "display values do not need full precision")]
pub fn precise(value: u64) -> f64 {
    value as f64
}

#[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "non-negative rates")]
pub fn whole(value: f64) -> u64 {
    value.max(0.0) as u64
}
