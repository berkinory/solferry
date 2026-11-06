use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use beam_proto::{
    Entry, FileResult, FileStart, Finalized, HASH_LEN, close, read_frame, write_frame,
};
use bytes::{Bytes, BytesMut};
use futures::{StreamExt, TryStreamExt, stream};
use iroh::endpoint::{Connection, RecvStream};
use tokio::sync::mpsc;

use super::{CHUNK_SIZE, Manifest, Meter};
use crate::error::{Error, Result, StorageContext};
use crate::route::{Route, current_route};

const FILE_CONCURRENCY: usize = 16;
const READ_AHEAD: usize = 4;

pub(crate) async fn run(
    connection: &Connection,
    control: &mut RecvStream,
    manifest: &Arc<Manifest>,
    offsets: &[u64],
    meter: &Meter,
) -> Result<Route> {
    let files: Vec<usize> = (0..manifest.entries.len())
        .filter(|&index| !manifest.entries[index].is_dir)
        .collect();
    let count = files.len();
    let uploads = stream::iter(files)
        .map(|index| send_file(connection, manifest, index, offsets[index], meter))
        .buffer_unordered(FILE_CONCURRENCY)
        .try_collect::<()>();
    tokio::try_join!(uploads, confirm(control, manifest, count))?;
    let route = current_route(connection);
    connection.close(close::DONE.into(), b"done");
    Ok(route)
}

async fn confirm(control: &mut RecvStream, manifest: &Manifest, count: usize) -> Result<()> {
    for _ in 0..count {
        let result: FileResult = read_frame(control).await?;
        if !result.ok {
            let entry = manifest.entries.get(result.index as usize);
            return Err(Error::HashMismatch(
                entry.map(|e| e.path.clone()).unwrap_or_default(),
            ));
        }
    }
    let finalized: Finalized = read_frame(control).await?;
    if finalized.ok {
        Ok(())
    } else {
        Err(Error::ConnectionLost)
    }
}

async fn send_file(
    connection: &Connection,
    manifest: &Arc<Manifest>,
    index: usize,
    offset: u64,
    meter: &Meter,
) -> Result<()> {
    let source: PathBuf = manifest.sources[index].clone();
    let Entry { size, .. } = manifest.entries[index];
    let (chunks, mut incoming) = mpsc::channel::<Bytes>(READ_AHEAD);
    let reader = tokio::task::spawn_blocking(move || read_source(&source, size, offset, &chunks));

    let mut stream = connection
        .open_uni()
        .await
        .map_err(|_| Error::ConnectionLost)?;
    let start = FileStart {
        index: u32::try_from(index).unwrap_or(u32::MAX),
        offset,
    };
    write_frame(&mut stream, &start).await?;
    while let Some(chunk) = incoming.recv().await {
        let len = chunk.len() as u64;
        stream
            .write_chunk(chunk)
            .await
            .map_err(|_| Error::ConnectionLost)?;
        meter.add(len);
    }
    let hash = reader
        .await
        .map_err(|error| Error::Protocol(error.to_string()))??;
    let bytes: &[u8; HASH_LEN] = hash.as_bytes();
    stream
        .write_all(bytes)
        .await
        .map_err(|_| Error::ConnectionLost)?;
    stream.finish().map_err(|_| Error::ConnectionLost)
}

/// Hashes the whole file so the receiver can verify resumed data, sending only bytes past `offset`.
fn read_source(
    path: &Path,
    size: u64,
    offset: u64,
    chunks: &mpsc::Sender<Bytes>,
) -> Result<blake3::Hash> {
    let mut file = File::open(path).at(path)?;
    let mut hasher = blake3::Hasher::new();
    let mut position = 0u64;
    loop {
        let mut buffer = BytesMut::zeroed(CHUNK_SIZE);
        let read = file.read(&mut buffer).at(path)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
        let start =
            usize::try_from(offset.saturating_sub(position)).map_or(read, |skip| skip.min(read));
        position += read as u64;
        if position > size {
            return Err(Error::SourceChanged(path.to_owned()));
        }
        if start < read {
            let chunk = buffer.freeze().slice(start..read);
            if chunks.blocking_send(chunk).is_err() {
                return Err(Error::ConnectionLost);
            }
        }
    }
    if position == size {
        Ok(hasher.finalize())
    } else {
        Err(Error::SourceChanged(path.to_owned()))
    }
}
