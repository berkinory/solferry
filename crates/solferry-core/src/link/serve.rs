use std::convert::Infallible;
use std::fs::File;
use std::net::Ipv4Addr;
use std::path::Path;
use std::sync::Arc;

use bytes::Bytes;
use data_encoding::BASE32_NOPAD;
use http_body_util::combinators::BoxBody;
use http_body_util::{BodyExt, Full, StreamBody};
use hyper::body::{Frame, Incoming};
use hyper::header::{CONTENT_DISPOSITION, CONTENT_LENGTH, CONTENT_TYPE};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use rand::Rng;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio::task::JoinSet;

use super::files::Served;
use super::{Link, Mode, page};
use crate::error::{Error, Result};
use crate::event::Event;
use crate::route;
use crate::transfer::free_path;
use crate::transfer::paths::sanitize;

/// A stable port next to the one transfers use, so a firewall rule can name it.
const PORT: u16 = 53320;
const CHUNK: usize = 256 * 1024;

type Body = BoxBody<Bytes, std::io::Error>;

struct State {
    token: String,
    mode: Mode,
    events: broadcast::Sender<Event>,
}

/// Serves `mode` to browsers on this device's local networks until the returned link is dropped.
pub(crate) async fn start(mode: Mode, events: broadcast::Sender<Event>) -> Result<Link> {
    let addresses = route::local_ipv4s();
    if addresses.is_empty() {
        return Err(Error::NoLocalNetwork);
    }
    let listener = match TcpListener::bind((Ipv4Addr::UNSPECIFIED, PORT)).await {
        Ok(listener) => listener,
        Err(_) => TcpListener::bind((Ipv4Addr::UNSPECIFIED, 0))
            .await
            .map_err(|error| Error::NetworkStart(error.to_string()))?,
    };
    let port =
        listener.local_addr().map_err(|error| Error::NetworkStart(error.to_string()))?.port();
    let mut secret = [0; 10];
    rand::rng().fill_bytes(&mut secret);
    let token = BASE32_NOPAD.encode(&secret).to_ascii_lowercase();
    let urls = addresses.iter().map(|ip| format!("http://{ip}:{port}/{token}/")).collect();
    let state = Arc::new(State { token, mode, events });
    let server = tokio::spawn(accept(listener, state)).abort_handle();
    Ok(Link { urls, server })
}

/// Connections live in a set owned here, so stopping the link also ends transfers in flight.
async fn accept(listener: TcpListener, state: Arc<State>) {
    let mut connections = JoinSet::new();
    loop {
        tokio::select! {
            accepted = listener.accept() => {
                let Ok((stream, from)) = accepted else { continue };
                if !route::shares_network(from.ip()) {
                    continue;
                }
                let state = state.clone();
                connections.spawn(async move {
                    let service = service_fn(move |request| respond(state.clone(), request));
                    let _ = http1::Builder::new().serve_connection(TokioIo::new(stream), service).await;
                });
            }
            Some(_) = connections.join_next(), if !connections.is_empty() => {}
        }
    }
}

async fn respond(
    state: Arc<State>,
    request: Request<Incoming>,
) -> Result<Response<Body>, Infallible> {
    let path = request.uri().path().to_owned();
    let Some(rest) =
        path.strip_prefix('/').and_then(|path| path.strip_prefix(state.token.as_str()))
    else {
        return Ok(plain(StatusCode::NOT_FOUND, "Not found"));
    };
    let response = match (request.method(), &state.mode, rest) {
        (&Method::GET, Mode::Share(files), "/") => html(page::share(files)),
        (&Method::GET, Mode::Receive(_), "/") => html(page::receive()),
        (&Method::GET, Mode::Share(files), rest) => match download_index(rest, files.len()) {
            Some(index) => download(&files[index]),
            None => plain(StatusCode::NOT_FOUND, "Not found"),
        },
        (&Method::PUT, Mode::Receive(folder), rest) => match rest.strip_prefix("/u/").map(decode) {
            Some(Some(name)) => upload(&state, folder, &name, request.into_body()).await,
            _ => plain(StatusCode::BAD_REQUEST, "Bad file name"),
        },
        _ => plain(StatusCode::NOT_FOUND, "Not found"),
    };
    Ok(response)
}

fn download_index(rest: &str, count: usize) -> Option<usize> {
    rest.strip_prefix("/f/")?.parse().ok().filter(|index| *index < count)
}

fn download(served: &Served) -> Response<Body> {
    let (file, size) = (served.file.clone(), served.size);
    let chunks = futures::stream::try_unfold(0, move |offset| {
        let file = file.clone();
        async move {
            if offset >= size {
                return Ok(None);
            }
            let length = usize::try_from(size - offset).unwrap_or(CHUNK).min(CHUNK);
            let chunk = tokio::task::spawn_blocking(move || read_at(&file, offset, length))
                .await
                .map_err(std::io::Error::other)??;
            Ok(Some((Frame::data(Bytes::from(chunk)), offset + length as u64)))
        }
    });
    let name = served.name.rsplit('/').next().unwrap_or(&served.name);
    Response::builder()
        .header(CONTENT_TYPE, "application/octet-stream")
        .header(CONTENT_LENGTH, size)
        .header(CONTENT_DISPOSITION, disposition(name))
        .body(BodyExt::boxed(StreamBody::new(chunks)))
        .unwrap_or_else(|_| plain(StatusCode::INTERNAL_SERVER_ERROR, "Couldn't send the file"))
}

#[cfg(unix)]
fn read_at(file: &File, offset: u64, length: usize) -> std::io::Result<Vec<u8>> {
    let mut chunk = vec![0; length];
    std::os::unix::fs::FileExt::read_exact_at(file, &mut chunk, offset)?;
    Ok(chunk)
}

#[cfg(windows)]
fn read_at(file: &File, offset: u64, length: usize) -> std::io::Result<Vec<u8>> {
    let mut chunk = vec![0; length];
    let mut filled = 0;
    while filled < length {
        let read = std::os::windows::fs::FileExt::seek_read(
            file,
            &mut chunk[filled..],
            offset + filled as u64,
        )?;
        if read == 0 {
            return Err(std::io::ErrorKind::UnexpectedEof.into());
        }
        filled += read;
    }
    Ok(chunk)
}

/// Writes beside the target under a hidden name and renames once complete, so a cut upload
/// never looks like a finished file.
async fn upload(state: &State, folder: &Path, name: &str, mut body: Incoming) -> Response<Body> {
    let Some(name) = sanitize(name).filter(|path| path.components().count() == 1) else {
        return plain(StatusCode::BAD_REQUEST, "Bad file name");
    };
    let partial = folder.join(format!(".{}.solferry-link", name.display()));
    let written = async {
        tokio::fs::create_dir_all(folder).await?;
        let mut file = tokio::fs::File::create(&partial).await?;
        let mut bytes = 0u64;
        while let Some(frame) = body.frame().await {
            if let Ok(data) = frame.map_err(std::io::Error::other)?.into_data() {
                file.write_all(&data).await?;
                bytes += data.len() as u64;
            }
        }
        file.sync_all().await?;
        let target = free_path(folder.join(&name));
        tokio::fs::rename(&partial, &target).await?;
        Ok::<_, std::io::Error>((target, bytes))
    };
    match written.await {
        Ok((target, bytes)) => {
            let name = target.file_name().unwrap_or_default().to_string_lossy().into_owned();
            let _ = state.events.send(Event::LinkReceived { name, bytes });
            plain(StatusCode::CREATED, "Saved")
        }
        Err(error) => {
            let _ = tokio::fs::remove_file(&partial).await;
            tracing::warn!(%error, "link upload failed");
            plain(StatusCode::INTERNAL_SERVER_ERROR, "Couldn't save the file")
        }
    }
}

fn html(page: String) -> Response<Body> {
    Response::builder()
        .header(CONTENT_TYPE, "text/html; charset=utf-8")
        .body(full(page))
        .unwrap_or_else(|_| plain(StatusCode::INTERNAL_SERVER_ERROR, "Error"))
}

fn plain(status: StatusCode, text: &'static str) -> Response<Body> {
    let mut response = Response::new(full(text.to_owned()));
    *response.status_mut() = status;
    response
}

fn full(text: String) -> Body {
    Full::new(Bytes::from(text)).map_err(|never| match never {}).boxed()
}

/// Keeps non-ASCII names intact for browsers that read `filename*`.
fn disposition(name: &str) -> String {
    let ascii: String = name
        .chars()
        .map(|c| if (c.is_ascii_graphic() || c == ' ') && c != '"' && c != '\\' { c } else { '_' })
        .collect();
    format!("attachment; filename=\"{ascii}\"; filename*=UTF-8''{}", encode(name))
}

fn encode(text: &str) -> String {
    text.bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                (byte as char).to_string()
            }
            _ => format!("%{byte:02X}"),
        })
        .collect()
}

fn decode(text: &str) -> Option<String> {
    let mut bytes = Vec::with_capacity(text.len());
    let mut input = text.bytes();
    while let Some(byte) = input.next() {
        if byte == b'%' {
            let high = char::from(input.next()?).to_digit(16)?;
            let low = char::from(input.next()?).to_digit(16)?;
            bytes.push(u8::try_from(high * 16 + low).ok()?);
        } else {
            bytes.push(byte);
        }
    }
    String::from_utf8(bytes).ok()
}
