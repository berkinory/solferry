use iroh::{RelayConfig, RelayUrl};

use crate::error::{Error, Result};

/// A relay the way people and devices share it: `https://relay.example.com#token`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelayLink {
    url: RelayUrl,
    token: Option<String>,
}

impl RelayLink {
    pub fn parse(text: &str) -> Result<Self> {
        let text = text.trim();
        let (url, token) = text.split_once('#').unwrap_or((text, ""));
        let parsed = url.parse().map_err(|_| Error::InvalidRelayUrl(url.to_owned()))?;
        Ok(Self { url: parsed, token: (!token.is_empty()).then(|| token.to_owned()) })
    }

    #[must_use]
    pub fn url(&self) -> &RelayUrl {
        &self.url
    }

    /// The link with its token hidden, for screens and terminals: `https://relay.example.com#••••`.
    #[must_use]
    pub fn masked(&self) -> String {
        let address = self.url.to_string();
        let address = address.trim_end_matches('/');
        if self.token.is_some() { format!("{address}#••••") } else { address.to_owned() }
    }

    /// Whether both links name the same server, whatever their tokens.
    #[must_use]
    pub fn same_server(&self, other: &Self) -> bool {
        self.url == other.url
    }

    pub(crate) fn config(&self) -> RelayConfig {
        let config = RelayConfig::from(self.url.clone());
        match &self.token {
            Some(token) => config.with_auth_token(token.clone()),
            None => config,
        }
    }
}
