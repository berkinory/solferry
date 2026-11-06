use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use beam_proto::{Entry, FileResult, FileStart, Finalized, HASH_LEN, read_frame, write_frame};
use bytes::Bytes;
use iroh::endpoint::{Connection, RecvStream, SendStream};
use tokio::sync::mpsc;
use tokio::task::JoinSet;

use super::staging::write_part;
use super::{CHUNK_SIZE, Meter, Staging};
use crate::error::{Error, Result};

const CLOSE_WAIT: Duration = Duration::from_secs(5);
const WRITE_BEHIND: usize = 16;

#[derive(Debug)]
pub(crate) struct Plan {
    pub(crate) destination: PathBuf,
    pub(crate) entries: Vec<Entry>,
    pub(crate) targets: Vec<PathBuf>,
    pub(crate) offsets: Vec<u64>,
    pub(crate) staging: Staging,
}

pub(crate) async fn run(
    connection: &Connection,
    control: &mut SendStream,
    plan: Plan,
    meter: Arc<Meter>,
) -> Result<Vec<PathBuf>> {
    let plan = Arc::new(plan);
    let needed = plan.entries.iter().filter(|entry| !entry.is_dir).count();
    let (results, mut pending) = mpsc::channel::<FileResult>(32);
    let mut files = JoinSet::new();
    let mut accepted = 0;
    let mut completed = 0;
    let mut seen = vec![false; plan.entries.len()];

    while completed < needed {
        tokio::select! {
            stream = connection.accept_uni(), if accepted < needed => {
                let mut stream = stream.map_err(|_| Error::ConnectionLost)?;
                let start: FileStart = read_frame(&mut stream).await?;
                let index = claim(&plan, &mut seen, &start)?;
                accepted += 1;
                let (plan, meter, results) = (plan.clone(), meter.clone(), results.clone());
                files.spawn(async move { receive_file(stream, &plan, index, &meter, &results).await });
            }
            Some(result) = pending.recv() => write_frame(control, &result).await?,
            Some(joined) = files.join_next() => {
                joined.map_err(|error| Error::Protocol(error.to_string()))??;
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
    .map_err(|error| Error::Protocol(error.to_string()))??;
    write_frame(control, &Finalized { ok: true }).await?;
    let _ = tokio::time::timeout(CLOSE_WAIT, connection.closed()).await;
    Ok(delivered)
}

fn claim(plan: &Plan, seen: &mut [bool], start: &FileStart) -> Result<usize> {
    let index = start.index as usize;
    let valid = plan.entries.get(index).is_some_and(|entry| !entry.is_dir)
        && plan.offsets[index] == start.offset
        && !seen[index];
    if !valid {
        return Err(Error::Protocol(format!("unexpected file stream {index}")));
    }
    seen[index] = true;
    Ok(index)
}

async fn receive_file(
    mut stream: RecvStream,
    plan: &Plan,
    index: usize,
    meter: &Meter,
    results: &mpsc::Sender<FileResult>,
) -> Result<()> {
    let offset = plan.offsets[index];
    let part = plan.staging.part(index);
    let (chunks, incoming) = mpsc::channel::<Bytes>(WRITE_BEHIND);
    let writer = {
        let part = part.clone();
        tokio::task::spawn_blocking(move || write_part(&part, offset, incoming))
    };
    let mut remaining = plan.entries[index].size - offset;
    while remaining > 0 {
        let limit = usize::try_from(remaining).unwrap_or(CHUNK_SIZE).min(CHUNK_SIZE);
        let chunk = stream.read_chunk(limit).await.map_err(|_| Error::ConnectionLost)?;
        let chunk = chunk.ok_or(Error::ConnectionLost)?;
        remaining -= chunk.len() as u64;
        meter.add(chunk.len() as u64);
        if chunks.send(chunk).await.is_err() {
            break;
        }
    }
    let mut expected = [0; HASH_LEN];
    let hash_read = stream.read_exact(&mut expected).await;
    drop(chunks);
    let actual = writer.await.map_err(|error| Error::Protocol(error.to_string()))??;
    hash_read.map_err(|_| Error::ConnectionLost)?;

    let ok = actual == blake3::Hash::from_bytes(expected);
    if !ok {
        discard(&part);
    }
    let index_wire = u32::try_from(index).unwrap_or(u32::MAX);
    let _ = results.send(FileResult { index: index_wire, ok }).await;
    if ok { Ok(()) } else { Err(Error::HashMismatch(plan.entries[index].path.clone())) }
}

fn discard(part: &Path) {
    let _ = std::fs::remove_file(part);
}
