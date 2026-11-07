use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use iroh::endpoint::{Connection, RecvStream, SendStream};
use solferry_proto::{Entry, FileResult, FileStart, Finalized, HASH_LEN, read_frame, write_frame};
use tokio::sync::mpsc;
use tokio::task::JoinSet;

use super::segments::{Progress, segments};
use super::staging::{Segment, forget, write_part};
use super::{CHUNK_SIZE, Meter, Staging};
use crate::error::{Error, Result};

const CLOSE_WAIT: Duration = Duration::from_secs(5);
const WRITE_BEHIND: usize = 16;

#[derive(Debug)]
pub(crate) struct Plan {
    pub(crate) destination: PathBuf,
    pub(crate) entries: Vec<Entry>,
    pub(crate) targets: Vec<PathBuf>,
    pub(crate) progress: Progress,
    pub(crate) staging: Staging,
}

pub(crate) async fn run(
    connection: &Connection,
    control: &mut SendStream,
    plan: Plan,
    meter: Arc<Meter>,
) -> Result<Vec<PathBuf>> {
    let plan = Arc::new(plan);
    let needed: usize = plan.progress.iter().map(Vec::len).sum();
    let (results, mut pending) = mpsc::channel::<FileResult>(32);
    let mut files = JoinSet::new();
    let mut accepted = 0;
    let mut completed = 0;
    let mut seen = HashSet::new();

    while completed < needed {
        tokio::select! {
            stream = connection.accept_uni(), if accepted < needed => {
                let mut stream = stream.map_err(|_| Error::ConnectionLost)?;
                let start: FileStart = read_frame(&mut stream).await?;
                let segment = claim(&plan, &mut seen, &start)?;
                accepted += 1;
                let (meter, results) = (meter.clone(), results.clone());
                let name = plan.entries[start.index as usize].path.clone();
                let result = FileResult { index: start.index, ok: true, segment: start.segment };
                files.spawn(async move {
                    receive_segment(stream, segment, name, result, &meter, &results).await
                });
            }
            Some(result) = pending.recv() => write_frame(control, &result).await?,
            Some(joined) = files.join_next() => {
                if let Err(error) = joined.map_err(|error| Error::Protocol(error.to_string()))? {
                    drop(results);
                    while let Some(result) = pending.recv().await {
                        write_frame(control, &result).await?;
                    }
                    let _ = control.finish();
                    let _ = tokio::time::timeout(CLOSE_WAIT, connection.closed()).await;
                    return Err(error);
                }
                completed += 1;
            }
        }
    }
    drop(results);
    while let Some(result) = pending.recv().await {
        write_frame(control, &result).await?;
    }

    let finished = plan.clone();
    let delivered = tokio::task::spawn_blocking(move || {
        finished.staging.finalize(&finished.destination, &finished.entries, &finished.targets)
    })
    .await
    .map_err(|error| Error::Protocol(error.to_string()))?;
    write_frame(control, &Finalized { ok: delivered.is_ok() }).await?;
    let _ = tokio::time::timeout(CLOSE_WAIT, connection.closed()).await;
    delivered
}

/// Accepts a stream only for a segment that was offered, starting exactly where the receiver said
/// it stopped, and only once.
fn claim(plan: &Plan, seen: &mut HashSet<(u32, u32)>, start: &FileStart) -> Result<Segment> {
    let (index, number) = (start.index as usize, start.segment as usize);
    let range = plan.entries.get(index).and_then(|entry| segments(entry).get(number).cloned());
    let held = plan.progress.get(index).and_then(|held| held.get(number)).copied();
    let (Some(range), Some(held)) = (range, held) else {
        return Err(Error::Protocol(format!("unexpected file stream {index}.{number}")));
    };
    if range.start + held != start.offset || !seen.insert((start.index, start.segment)) {
        return Err(Error::Protocol(format!("unexpected file stream {index}.{number}")));
    }
    Ok(Segment {
        part: plan.staging.part(index),
        progress: plan.staging.progress_file(index, number),
        range,
        held,
    })
}

async fn receive_segment(
    mut stream: RecvStream,
    segment: Segment,
    name: String,
    result: FileResult,
    meter: &Meter,
    results: &mpsc::Sender<FileResult>,
) -> Result<()> {
    let remaining = segment.range.end - segment.range.start - segment.held;
    let segment = std::sync::Arc::new(segment);
    let (chunks, incoming) = mpsc::channel::<Bytes>(WRITE_BEHIND);
    let writer = {
        let segment = segment.clone();
        tokio::task::spawn_blocking(move || write_part(&segment, incoming))
    };
    let received = pump(&mut stream, remaining, &chunks, meter).await;
    drop(chunks);
    let actual = writer.await.map_err(|error| Error::Protocol(error.to_string()))??;
    let expected = received?;

    let ok = actual == expected;
    if !ok {
        forget(&segment);
    }
    let _ = results.send(FileResult { ok, ..result }).await;
    if ok { Ok(()) } else { Err(Error::HashMismatch(name)) }
}

async fn pump(
    stream: &mut RecvStream,
    mut remaining: u64,
    chunks: &mpsc::Sender<Bytes>,
    meter: &Meter,
) -> Result<blake3::Hash> {
    while remaining > 0 {
        let limit = usize::try_from(remaining).unwrap_or(CHUNK_SIZE).min(CHUNK_SIZE);
        let chunk = stream.read_chunk(limit).await.map_err(|_| Error::ConnectionLost)?;
        let chunk = chunk.ok_or(Error::ConnectionLost)?;
        remaining -= chunk.len() as u64;
        meter.add(chunk.len() as u64);
        chunks.send(chunk).await.map_err(|_| Error::ConnectionLost)?;
    }
    let mut hash = [0; HASH_LEN];
    stream.read_exact(&mut hash).await.map_err(|_| Error::ConnectionLost)?;
    Ok(blake3::Hash::from_bytes(hash))
}
