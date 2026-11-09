//! Finding and fetching new releases. A manifest only counts if it is signed by the key built into
//! this binary, so GitHub is a place to download from and not something to trust.
//!
//! Builds without `SOLFERRY_UPDATE_KEY` never check, which keeps self-compiled and packaged copies
//! away from the official update channel.

use std::collections::HashMap;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use ed25519_dalek::{Signature, VerifyingKey};
use semver::Version;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use crate::error::{Error, Result};

const VERIFYING_KEY: Option<&str> = option_env!("SOLFERRY_UPDATE_KEY");
const MANIFEST_URL: &str = match option_env!("SOLFERRY_UPDATE_URL") {
    Some(url) => url,
    None => "https://github.com/berkinory/solferry/releases/latest/download/latest.json",
};
const MANIFEST_LIMIT: u64 = 64 * 1024;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
const CHECK_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct Asset {
    pub url: String,
    pub sha256: String,
    pub size: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct Release {
    pub version: String,
    /// The release's page, for platforms that can't install by themselves.
    pub page: String,
    assets: HashMap<String, Asset>,
}

impl Release {
    /// What this platform and architecture would install.
    #[must_use]
    pub fn asset(&self) -> Option<&Asset> {
        self.assets.get(&format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH))
    }
}

/// Whether this build was given a key to check releases with.
#[must_use]
pub const fn enabled() -> bool {
    VERIFYING_KEY.is_some()
}

fn failed(reason: impl std::fmt::Display) -> Error {
    Error::Update(reason.to_string())
}

fn client() -> Result<reqwest::Client> {
    let roots = rustls::RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let tls = rustls::ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(failed)?
        .with_root_certificates(roots)
        .with_no_client_auth();
    reqwest::Client::builder()
        .tls_backend_preconfigured(tls)
        .connect_timeout(CONNECT_TIMEOUT)
        .build()
        .map_err(failed)
}

async fn small_file(client: &reqwest::Client, url: &str) -> Result<Vec<u8>> {
    let mut response = client
        .get(url)
        .timeout(CHECK_TIMEOUT)
        .send()
        .await
        .and_then(reqwest::Response::error_for_status)
        .map_err(failed)?;
    let mut body = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(failed)? {
        body.extend_from_slice(&chunk);
        if body.len() as u64 > MANIFEST_LIMIT {
            return Err(failed("the update manifest is too large"));
        }
    }
    Ok(body)
}

/// The manifest, if its signature holds. `signature` is the hex of a raw Ed25519 signature.
fn verified(manifest: &[u8], signature: &[u8], key: &str) -> Result<Release> {
    let key: [u8; 32] = hex::decode(key)
        .ok()
        .and_then(|raw| raw.try_into().ok())
        .ok_or_else(|| failed("this build carries an invalid update key"))?;
    let key = VerifyingKey::from_bytes(&key).map_err(failed)?;
    let signature: [u8; 64] = hex::decode(signature.trim_ascii())
        .ok()
        .and_then(|raw| raw.try_into().ok())
        .ok_or_else(|| failed("the update signature is malformed"))?;
    key.verify_strict(manifest, &Signature::from_bytes(&signature))
        .map_err(|_| failed("the update signature doesn't match"))?;
    serde_json::from_slice(manifest).map_err(failed)
}

fn newer(release: &str, current: &str) -> Result<bool> {
    let release = Version::parse(release.trim_start_matches('v')).map_err(failed)?;
    let current = Version::parse(current).map_err(failed)?;
    Ok(release > current)
}

/// Asks the release channel for something newer than this build. `None` means up to date.
pub async fn check() -> Result<Option<Release>> {
    let Some(key) = VERIFYING_KEY else { return Ok(None) };
    let client = client()?;
    let manifest = small_file(&client, MANIFEST_URL).await?;
    let signature = small_file(&client, &format!("{MANIFEST_URL}.sig")).await?;
    let release = verified(&manifest, &signature, key)?;
    Ok(newer(&release.version, env!("CARGO_PKG_VERSION"))?.then_some(release))
}

/// Whether the file at `path` is exactly what the signed manifest describes.
pub async fn matches(asset: &Asset, path: &Path) -> bool {
    let (path, expected) = (path.to_owned(), asset.sha256.clone());
    tokio::task::spawn_blocking(move || {
        let Ok(mut file) = std::fs::File::open(path) else { return false };
        let mut hasher = Sha256::new();
        let mut buffer = vec![0; 1 << 20];
        loop {
            match file.read(&mut buffer) {
                Ok(0) => break,
                Ok(read) => hasher.update(&buffer[..read]),
                Err(_) => return false,
            }
        }
        hex::encode(hasher.finalize()).eq_ignore_ascii_case(&expected)
    })
    .await
    .unwrap_or(false)
}

/// Saves the asset to `destination`, which only appears once its hash has been checked.
pub async fn download(
    asset: &Asset,
    destination: &Path,
    mut progress: impl FnMut(u64, u64),
) -> Result<()> {
    let part = destination.with_extension("part");
    let result = fetch(asset, &part, &mut progress).await;
    if result.is_err() {
        let _ = tokio::fs::remove_file(&part).await;
    }
    result?;
    tokio::fs::rename(&part, destination).await.map_err(failed)
}

async fn fetch(asset: &Asset, part: &Path, progress: &mut impl FnMut(u64, u64)) -> Result<()> {
    let mut response = client()?
        .get(&asset.url)
        .send()
        .await
        .and_then(reqwest::Response::error_for_status)
        .map_err(failed)?;
    let mut file = File::create(part).await.map_err(failed)?;
    let mut hasher = Sha256::new();
    let mut received = 0;
    while let Some(chunk) = response.chunk().await.map_err(failed)? {
        received += chunk.len() as u64;
        if received > asset.size {
            return Err(failed("the update is larger than announced"));
        }
        hasher.update(&chunk);
        file.write_all(&chunk).await.map_err(failed)?;
        progress(received, asset.size);
    }
    file.sync_all().await.map_err(failed)?;
    if hex::encode(hasher.finalize()).eq_ignore_ascii_case(&asset.sha256) {
        Ok(())
    } else {
        Err(failed("the downloaded update is damaged"))
    }
}

#[cfg(test)]
mod tests {
    use ed25519_dalek::{Signer, SigningKey};

    use super::*;

    const MANIFEST: &[u8] = br#"{"version":"9.0.0","page":"https://example.com/9","assets":{}}"#;

    fn sign(body: &[u8], seed: u8) -> (String, Vec<u8>) {
        let key = SigningKey::from_bytes(&[seed; 32]);
        let signature = hex::encode(key.sign(body).to_bytes()).into_bytes();
        (hex::encode(key.verifying_key().to_bytes()), signature)
    }

    #[test]
    fn a_signed_manifest_is_believed() {
        let (key, signature) = sign(MANIFEST, 1);
        assert_eq!(verified(MANIFEST, &signature, &key).unwrap().version, "9.0.0");
    }

    #[test]
    fn a_changed_manifest_is_refused() {
        let (key, signature) = sign(MANIFEST, 1);
        let forged = String::from_utf8_lossy(MANIFEST).replace("9.0.0", "9.0.1");
        assert!(verified(forged.as_bytes(), &signature, &key).is_err());
    }

    #[test]
    fn a_signature_from_another_key_is_refused() {
        let (key, _) = sign(MANIFEST, 1);
        let (_, signature) = sign(MANIFEST, 2);
        assert!(verified(MANIFEST, &signature, &key).is_err());
    }

    #[test]
    fn only_later_versions_count_as_newer() {
        assert!(newer("0.10.0", "0.9.9").unwrap());
        assert!(newer("v1.0.0", "0.9.9").unwrap());
        assert!(!newer("0.9.9", "0.9.9").unwrap());
        assert!(!newer("0.9.8", "0.9.9").unwrap());
    }
}
