use std::ops::Range;
use std::sync::Arc;

use bytes::{Bytes, BytesMut};
use futures::{StreamExt, TryStreamExt, stream};
use iroh::endpoint::{Connection, RecvStream};
use solferry_proto::{FileResult, FileStart, Finalized, HASH_LEN, close, read_frame, write_frame};
use tokio::sync::mpsc;

use super::segments::{Progress, segments};
use super::source::{Source, read_at};
use super::{CHUNK_SIZE, Manifest, Meter, Throttle};
use crate::error::{Error, Result, StorageContext};
use crate::route::{Route, current_route};

/// Open streams at once: enough to fill the connection window, under the peer's stream limit.
const STREAM_CONCURRENCY: usize = 32;
const READ_AHEAD: usize = 4;

struct Unit {
    index: usize,
    segment: usize,
    range: Range<u64>,
    held: u64,
}

fn units(manifest: &Manifest, progress: &Progress) -> Vec<Unit> {
    manifest
        .entries
        .iter()
        .enumerate()
        .flat_map(|(index, entry)| {
            segments(entry).into_iter().enumerate().map(move |(segment, range)| Unit {
                index,
                segment,
                range,
                held: progress[index][segment],
            })
        })
        .collect()
}

pub(crate) async fn run(
    connection: &Connection,
    control: &mut RecvStream,
    manifest: &Arc<Manifest>,
    progress: &Progress,
    meter: &Meter,
    throttle: &Throttle,
) -> Result<Route> {
    let units = units(manifest, progress);
    let count = units.len();
    let uploads = stream::iter(units)
        .map(|unit| send_segment(connection, manifest, unit, meter, throttle))
        .buffer_unordered(STREAM_CONCURRENCY)
        .try_collect::<()>();
    tokio::try_join!(uploads, confirm(control, manifest, count))?;
    let route = current_route(connection);
    connection.close(close::DONE.into(), b"done");
    Ok(route)
}

/// Reads every segment's result before giving up on a damaged one, so the receiver gets to check
/// and forget all of them.
async fn confirm(control: &mut RecvStream, manifest: &Manifest, count: usize) -> Result<()> {
    let mut damaged = None;
    for _ in 0..count {
        let result: FileResult = read_frame(control).await?;
        if !result.ok && damaged.is_none() {
            let entry = manifest.entries.get(result.index as usize);
            damaged = Some(entry.map(|e| e.path.clone()).unwrap_or_default());
        }
    }
    if let Some(name) = damaged {
        return Err(Error::HashMismatch(name));
    }
    let finalized: Finalized = read_frame(control).await?;
    if finalized.ok { Ok(()) } else { Err(Error::DeliveryFailed) }
}

async fn send_segment(
    connection: &Connection,
    manifest: &Arc<Manifest>,
    unit: Unit,
    meter: &Meter,
    throttle: &Throttle,
) -> Result<()> {
    let source = manifest.sources[unit.index].clone();
    let size = manifest.entries[unit.index].size;
    let (range, held) = (unit.range.clone(), unit.held);
    let (chunks, mut incoming) = mpsc::channel::<Bytes>(READ_AHEAD);
    let reader =
        tokio::task::spawn_blocking(move || read_source(&source, size, range, held, &chunks));

    let mut stream = connection.open_uni().await.map_err(|_| Error::ConnectionLost)?;
    let start = FileStart {
        index: u32::try_from(unit.index).unwrap_or(u32::MAX),
        offset: unit.range.start + unit.held,
        segment: u32::try_from(unit.segment).unwrap_or(u32::MAX),
    };
    write_frame(&mut stream, &start).await?;
    while let Some(chunk) = incoming.recv().await {
        let len = chunk.len() as u64;
        throttle.pass(chunk.len()).await;
        stream.write_chunk(chunk).await.map_err(|_| Error::ConnectionLost)?;
        meter.add(len);
    }
    let hash = reader.await.map_err(|error| Error::Protocol(error.to_string()))??;
    let bytes: &[u8; HASH_LEN] = hash.as_bytes();
    stream.write_all(bytes).await.map_err(|_| Error::ConnectionLost)?;
    stream.finish().map_err(|_| Error::ConnectionLost)
}

/// Hashes the whole segment so the receiver can verify resumed data, sending only bytes past
/// what it already holds.
fn read_source(
    source: &Source,
    size: u64,
    range: Range<u64>,
    held: u64,
    chunks: &mpsc::Sender<Bytes>,
) -> Result<blake3::Hash> {
    let path = source.label();
    let file = source.reader().at(path)?;
    if file.metadata().at(path)?.len() != size {
        return Err(Error::SourceChanged(path.to_owned()));
    }
    let mut hasher = blake3::Hasher::new();
    let mut position = 0u64;
    let length = range.end - range.start;
    while position < length {
        let wanted = usize::try_from(length - position).unwrap_or(CHUNK_SIZE).min(CHUNK_SIZE);
        let mut buffer = BytesMut::zeroed(wanted);
        let read = read_at(&file, &mut buffer, range.start + position).at(path)?;
        if read == 0 {
            return Err(Error::SourceChanged(path.to_owned()));
        }
        hasher.update(&buffer[..read]);
        let skip =
            usize::try_from(held.saturating_sub(position)).map_or(read, |skip| skip.min(read));
        position += read as u64;
        if skip < read && chunks.blocking_send(buffer.freeze().slice(skip..read)).is_err() {
            return Err(Error::ConnectionLost);
        }
    }
    Ok(hasher.finalize())
}
