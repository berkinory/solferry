use std::ops::Range;

use solferry_proto::Entry;

const MIB: u64 = 1024 * 1024;
/// One QUIC stream moves at most its 2 MiB window per round trip, so large files travel as
/// several streams at once. Both sides derive the same split from the file size alone.
const MAX_SEGMENTS: u64 = 16;
const MIN_SEGMENT: u64 = 8 * MIB;

/// Bytes already held for each segment of each entry, in offer order.
pub(crate) type Progress = Vec<Vec<u64>>;

pub(crate) fn segments(entry: &Entry) -> Vec<Range<u64>> {
    if entry.is_dir {
        return Vec::new();
    }
    if entry.size == 0 {
        return std::iter::once(0..0).collect();
    }
    let length = entry.size.div_ceil(MAX_SEGMENTS).max(MIN_SEGMENT).next_multiple_of(MIB);
    let mut ranges = Vec::new();
    let mut start = 0;
    while start < entry.size {
        let end = (start + length).min(entry.size);
        ranges.push(start..end);
        start = end;
    }
    ranges
}

pub(crate) fn flatten(progress: &Progress) -> Vec<u64> {
    progress.iter().flatten().copied().collect()
}

/// Splits a reply's flat table back into segments, or `None` if it does not fit the offer.
pub(crate) fn unflatten(entries: &[Entry], flat: &[u64]) -> Option<Progress> {
    let mut values = flat.iter().copied();
    let mut progress = Vec::with_capacity(entries.len());
    for entry in entries {
        let mut held = Vec::new();
        for range in segments(entry) {
            let value = values.next()?;
            if value > range.end - range.start {
                return None;
            }
            held.push(value);
        }
        progress.push(held);
    }
    values.next().is_none().then_some(progress)
}

pub(crate) fn total(progress: &Progress) -> u64 {
    progress.iter().flatten().sum()
}
