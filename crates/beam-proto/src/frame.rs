use prost::Message;
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub const MAX_FRAME_LEN: u32 = 64 * 1024 * 1024;
/// Limit for messages read before a peer is trusted, which are always small.
pub const MAX_SMALL_FRAME_LEN: u32 = 64 * 1024;

#[derive(Debug, Error)]
pub enum FrameError {
    #[error("stream failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("frame of {0} bytes exceeds the limit")]
    TooLarge(u32),
    #[error("malformed message: {0}")]
    Decode(#[from] prost::DecodeError),
}

pub async fn write_frame<W, M>(writer: &mut W, message: &M) -> Result<(), FrameError>
where
    W: AsyncWrite + Unpin,
    M: Message,
{
    let body = message.encode_to_vec();
    let len = u32::try_from(body.len()).unwrap_or(u32::MAX);
    if len > MAX_FRAME_LEN {
        return Err(FrameError::TooLarge(len));
    }
    writer.write_u32(len).await?;
    writer.write_all(&body).await?;
    Ok(())
}

pub async fn read_frame<R, M>(reader: &mut R) -> Result<M, FrameError>
where
    R: AsyncRead + Unpin,
    M: Message + Default,
{
    read_limited(reader, MAX_FRAME_LEN).await
}

pub async fn read_small_frame<R, M>(reader: &mut R) -> Result<M, FrameError>
where
    R: AsyncRead + Unpin,
    M: Message + Default,
{
    read_limited(reader, MAX_SMALL_FRAME_LEN).await
}

async fn read_limited<R, M>(reader: &mut R, limit: u32) -> Result<M, FrameError>
where
    R: AsyncRead + Unpin,
    M: Message + Default,
{
    let len = reader.read_u32().await?;
    if len > limit {
        return Err(FrameError::TooLarge(len));
    }
    let mut body = vec![0; len as usize];
    reader.read_exact(&mut body).await?;
    Ok(M::decode(body.as_slice())?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Hello;

    #[tokio::test]
    async fn rejects_oversized_length_before_allocating() {
        let wire = (MAX_FRAME_LEN + 1).to_be_bytes();
        let result = read_frame::<_, Hello>(&mut wire.as_slice()).await;
        assert!(matches!(result, Err(FrameError::TooLarge(_))));
    }
}
