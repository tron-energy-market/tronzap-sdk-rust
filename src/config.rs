//! Client configuration.

use std::fmt;
use std::time::Duration;

use reqwest::header::HeaderValue;

use crate::client::TronzapClient;
use crate::error::{Result, TransportError, TronzapError, ValidationError};

/// The production API endpoint.
pub const DEFAULT_BASE_URL: &str = "https://api.tronzap.com";

/// The timeout applied to each request unless
/// [`TronzapClientBuilder::timeout`] overrides it.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// Builds a [`TronzapClient`].
///
/// The API token and secret are required; everything else has a default.
///
/// ```
/// use std::time::Duration;
/// use tronzap_sdk::TronzapClient;
///
/// let client = TronzapClient::builder()
///     .api_token("your_api_token")
///     .api_secret("your_api_secret")
///     .base_url("api.tronzap.com")
///     .timeout(Duration::from_secs(10))
///     .user_agent("my-app/1.0")
///     .build()?;
/// # Ok::<(), tronzap_sdk::TronzapError>(())
/// ```
#[derive(Default)]
#[must_use]
pub struct TronzapClientBuilder {
    api_token: Option<String>,
    api_secret: Option<String>,
    base_url: Option<String>,
    timeout: Option<Duration>,
    user_agent: Option<String>,
    http_client: Option<reqwest::Client>,
}

impl TronzapClientBuilder {
    /// Returns a builder with nothing set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the API token from your TronZap dashboard. It is sent as a bearer
    /// token. Required.
    pub fn api_token(mut self, api_token: impl Into<String>) -> Self {
        self.api_token = Some(api_token.into());
        self
    }

    /// Sets the API secret from your TronZap dashboard. It signs every request
    /// body and is never sent. Required.
    pub fn api_secret(mut self, api_secret: impl Into<String>) -> Self {
        self.api_secret = Some(api_secret.into());
        self
    }

    /// Points the client at another API host. Defaults to [`DEFAULT_BASE_URL`].
    ///
    /// A bare domain gets `https://`, and a trailing slash is trimmed, so
    /// `api.tronzap.com`, `api.tronzap.com/` and `https://api.tronzap.com` are
    /// equivalent. Give an explicit scheme to opt out, for example
    /// `http://localhost:8080` against a local mock.
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    /// Sets the timeout of each request, from connecting until the response body
    /// is read. Defaults to [`DEFAULT_TIMEOUT`].
    ///
    /// When set, it replaces the timeout of a client passed to
    /// [`http_client`](Self::http_client). When not set, such a client keeps its
    /// own timeout settings, including having none.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Overrides the `User-Agent` header, `tronzap-sdk-rust/<version>` by default.
    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    /// Sends requests through your own [`reqwest::Client`], for a proxy, custom
    /// certificates or connection pool settings. Use the [`reqwest`](crate::reqwest)
    /// re-exported by this crate so the versions match.
    ///
    /// The client is used as given: headers are set per request, and its timeout
    /// is replaced only when [`timeout`](Self::timeout) is called.
    ///
    /// ```
    /// use tronzap_sdk::{TronzapClient, reqwest};
    ///
    /// let http = reqwest::Client::builder().pool_max_idle_per_host(4).build()?;
    /// let client = TronzapClient::builder()
    ///     .api_token("your_api_token")
    ///     .api_secret("your_api_secret")
    ///     .http_client(http)
    ///     .build()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn http_client(mut self, http_client: reqwest::Client) -> Self {
        self.http_client = Some(http_client);
        self
    }

    /// Builds the client.
    ///
    /// # Errors
    ///
    /// Returns [`TronzapError::Validation`] when the token or secret is missing,
    /// the base URL is not an `http` or `https` URL, the timeout is zero, or a
    /// value cannot be sent in an HTTP header. Returns
    /// [`TronzapError::Transport`] when the default HTTP client cannot be created,
    /// for example because no TLS backend is available.
    pub fn build(self) -> Result<TronzapClient> {
        let api_token = required("api_token", self.api_token)?;
        let api_secret = required("api_secret", self.api_secret)?;

        let mut authorization = header("api_token", &format!("Bearer {api_token}"))?;
        authorization.set_sensitive(true);

        let base_url = normalize_base_url(self.base_url.as_deref().unwrap_or(DEFAULT_BASE_URL))?;

        if self.timeout.is_some_and(|t| t.is_zero()) {
            return Err(invalid("timeout", "must be positive"));
        }
        let timeout = match (&self.http_client, self.timeout) {
            (None, None) => Some(DEFAULT_TIMEOUT),
            (_, timeout) => timeout,
        };

        let user_agent = match self.user_agent {
            Some(ua) if ua.trim().is_empty() => return Err(invalid("user_agent", "must not be blank")),
            Some(ua) => header("user_agent", &ua)?,
            None => HeaderValue::from_static(DEFAULT_USER_AGENT),
        };

        let http = match self.http_client {
            Some(http) => http,
            None => reqwest::Client::builder()
                .build()
                .map_err(|e| TronzapError::Transport(TransportError::new(e)))?,
        };

        Ok(TronzapClient::from_config(Config {
            http,
            base_url,
            authorization,
            api_secret,
            user_agent,
            timeout,
        }))
    }
}

impl fmt::Debug for TronzapClientBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TronzapClientBuilder")
            .field("api_token", &self.api_token.as_ref().map(|_| "<redacted>"))
            .field("api_secret", &self.api_secret.as_ref().map(|_| "<redacted>"))
            .field("base_url", &self.base_url)
            .field("timeout", &self.timeout)
            .field("user_agent", &self.user_agent)
            .field("http_client", &self.http_client.is_some())
            .finish()
    }
}

const DEFAULT_USER_AGENT: &str = concat!("tronzap-sdk-rust/", env!("CARGO_PKG_VERSION"));

pub(crate) struct Config {
    pub(crate) http: reqwest::Client,
    pub(crate) base_url: String,
    pub(crate) authorization: HeaderValue,
    pub(crate) api_secret: String,
    pub(crate) user_agent: HeaderValue,
    pub(crate) timeout: Option<Duration>,
}

fn invalid(field: &'static str, message: &str) -> TronzapError {
    TronzapError::Validation(ValidationError::new(field, message))
}

fn required(field: &'static str, value: Option<String>) -> Result<String> {
    match value {
        Some(v) if !v.trim().is_empty() => Ok(v),
        _ => Err(invalid(field, "is required")),
    }
}

fn header(field: &'static str, value: &str) -> Result<HeaderValue> {
    HeaderValue::from_str(value)
        .map_err(|_| invalid(field, "contains characters not allowed in an HTTP header"))
}

fn normalize_base_url(raw: &str) -> Result<String> {
    let raw = raw.trim();
    if raw.trim_matches('/').is_empty() {
        return Err(invalid("base_url", "is required"));
    }
    let url = if raw.contains("://") {
        raw.trim_end_matches('/').to_owned()
    } else {
        format!("https://{}", raw.trim_matches('/'))
    };
    let parsed =
        reqwest::Url::parse(&url).map_err(|e| invalid("base_url", &format!("is not a valid URL: {e}")))?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err(invalid("base_url", "must use http or https"));
    }
    if parsed.host_str().is_none_or(str::is_empty) || parsed.query().is_some() || parsed.fragment().is_some()
    {
        return Err(invalid("base_url", "must be a host with an optional path"));
    }
    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field(result: Result<TronzapClient>) -> String {
        match result {
            Err(TronzapError::Validation(e)) => e.field().to_owned(),
            other => panic!("expected a validation error, got {other:?}"),
        }
    }

    fn builder() -> TronzapClientBuilder {
        TronzapClient::builder().api_token("token").api_secret("secret")
    }

    #[test]
    fn normalizes_base_urls() {
        for raw in ["api.tronzap.com", "api.tronzap.com/", " https://api.tronzap.com/ ", "//api.tronzap.com"]
        {
            assert_eq!(normalize_base_url(raw).unwrap(), "https://api.tronzap.com", "{raw}");
        }
        assert_eq!(normalize_base_url("http://localhost:8080").unwrap(), "http://localhost:8080");
        assert_eq!(normalize_base_url("https://host/prefix/").unwrap(), "https://host/prefix");
    }

    #[test]
    fn rejects_bad_base_urls() {
        for raw in
            ["", "  /", "ftp://host", "https://", "https://host?x=1", "https://host#top", "http://exa mple"]
        {
            assert!(normalize_base_url(raw).is_err(), "{raw}");
        }
    }

    #[test]
    fn requires_credentials() {
        assert_eq!(field(TronzapClient::builder().api_secret("s").build()), "api_token");
        assert_eq!(field(TronzapClient::builder().api_token("t").build()), "api_secret");
        assert_eq!(field(TronzapClient::builder().api_token(" ").api_secret("s").build()), "api_token");
    }

    #[test]
    fn rejects_bad_settings() {
        assert_eq!(field(builder().timeout(Duration::ZERO).build()), "timeout");
        assert_eq!(field(builder().user_agent("").build()), "user_agent");
        assert_eq!(field(builder().user_agent("bad\nagent").build()), "user_agent");
        assert_eq!(field(TronzapClient::builder().api_token("t\r\n").api_secret("s").build()), "api_token");
        assert_eq!(field(builder().base_url("ftp://x").build()), "base_url");
    }

    #[test]
    fn debug_redacts_credentials() {
        let builder = TronzapClient::builder().api_token("tok-XYZ").api_secret("sec-XYZ");
        let text = format!("{builder:?}");
        assert!(!text.contains("XYZ") && text.contains("<redacted>"), "{text}");
        let text = format!("{:?}", builder.build().unwrap());
        assert!(!text.contains("XYZ") && text.contains("<redacted>"), "{text}");
    }

    #[test]
    fn default_user_agent_has_version() {
        assert_eq!(DEFAULT_USER_AGENT, format!("tronzap-sdk-rust/{}", env!("CARGO_PKG_VERSION")));
    }
}
