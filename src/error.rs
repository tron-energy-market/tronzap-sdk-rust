//! Errors returned by the SDK.

use std::error::Error as StdError;
use std::fmt;
use std::time::Duration;

/// A `Result` whose error is [`TronzapError`].
pub type Result<T, E = TronzapError> = std::result::Result<T, E>;

/// Every way an SDK call can fail.
///
/// ```no_run
/// use tronzap_sdk::{ErrorCode, TronzapClient, TronzapError};
/// use tronzap_sdk::requests::EnergyTransactionRequest;
///
/// # async fn run(client: TronzapClient) {
/// let request = EnergyTransactionRequest::new("TRecipientAddress", 65000);
/// match client.create_energy_transaction(&request).await {
///     Ok(tx) => println!("transaction {} is {}", tx.id, tx.status),
///     Err(TronzapError::Api(e)) if e.error_code() == Some(ErrorCode::InsufficientFunds) => {
///         eprintln!("top up the account");
///     }
///     Err(TronzapError::Api(e)) if e.is_rate_limited() => {
///         eprintln!("retry after {:?}", e.retry_after());
///     }
///     Err(TronzapError::Authentication(e)) => eprintln!("check the token and secret: {e}"),
///     Err(TronzapError::Transport(e)) if e.is_timeout() => eprintln!("timed out, safe to retry"),
///     Err(e) => eprintln!("{e}"),
/// }
/// # }
/// ```
#[derive(Debug)]
#[non_exhaustive]
pub enum TronzapError {
    /// No response arrived: DNS failure, refused connection, TLS failure or timeout.
    Transport(TransportError),
    /// The API answered with an error: a non-zero `code`, or a non-2xx status.
    Api(ApiError),
    /// The API rejected the credentials: error code 1, or HTTP 401 or 403.
    Authentication(ApiError),
    /// The SDK rejected the arguments or the configuration; nothing was sent.
    Validation(ValidationError),
    /// A request could not be encoded, or a 2xx response could not be decoded.
    Serialization(SerializationError),
}

impl TronzapError {
    /// Returns the API error for [`TronzapError::Api`] and
    /// [`TronzapError::Authentication`].
    pub fn api_error(&self) -> Option<&ApiError> {
        match self {
            TronzapError::Api(e) | TronzapError::Authentication(e) => Some(e),
            _ => None,
        }
    }

    /// Returns `true` when the request timed out.
    pub fn is_timeout(&self) -> bool {
        matches!(self, TronzapError::Transport(e) if e.is_timeout())
    }
}

impl fmt::Display for TronzapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TronzapError::Transport(e) => e.fmt(f),
            TronzapError::Api(e) => write!(f, "tronzap api error: {e}"),
            TronzapError::Authentication(e) => write!(f, "tronzap authentication failed: {e}"),
            TronzapError::Validation(e) => e.fmt(f),
            TronzapError::Serialization(e) => e.fmt(f),
        }
    }
}

impl StdError for TronzapError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            TronzapError::Transport(e) => e.source(),
            TronzapError::Serialization(e) => e.source(),
            _ => None,
        }
    }
}

/// An error response from the API.
///
/// The API reports some failures with a 2xx status and others with 4xx or 5xx, so
/// a response body with a non-zero `code` is always an `ApiError` whatever its
/// status. A non-2xx response without a code is an `ApiError` too, with
/// [`ApiError::code`] returning `None`.
#[derive(Debug, Clone)]
pub struct ApiError(Box<ApiErrorData>);

#[derive(Debug, Clone)]
pub(crate) struct ApiErrorData {
    pub(crate) status: u16,
    pub(crate) code: Option<i64>,
    pub(crate) key: Option<String>,
    pub(crate) message: String,
    pub(crate) request_id: Option<String>,
    pub(crate) body: String,
    pub(crate) retry_after: Option<Duration>,
}

impl ApiError {
    pub(crate) fn new(data: ApiErrorData) -> Self {
        ApiError(Box::new(data))
    }

    /// Returns the HTTP status the response arrived with.
    pub fn status(&self) -> u16 {
        self.0.status
    }

    /// Returns the API error code, or `None` when the response carried none.
    pub fn code(&self) -> Option<i64> {
        self.0.code
    }

    /// Returns the error code as a known [`ErrorCode`], or `None` when the response
    /// carried no code or one this SDK version does not know.
    pub fn error_code(&self) -> Option<ErrorCode> {
        self.0.code.and_then(ErrorCode::from_code)
    }

    /// Returns the machine-readable error key, such as `invalid_tron_address` or
    /// `invalid_tron_address.from_address`.
    pub fn key(&self) -> Option<&str> {
        self.0.key.as_deref()
    }

    /// Returns the human-readable message.
    pub fn message(&self) -> &str {
        &self.0.message
    }

    /// Returns the identifier the API assigned to the request. Quote it when
    /// contacting support.
    pub fn request_id(&self) -> Option<&str> {
        self.0.request_id.as_deref()
    }

    /// Returns the raw response body.
    pub fn body(&self) -> &str {
        &self.0.body
    }

    /// Returns the delay from the `Retry-After` header, when the API sent one.
    pub fn retry_after(&self) -> Option<Duration> {
        self.0.retry_after
    }

    /// Returns `true` for HTTP 429.
    pub fn is_rate_limited(&self) -> bool {
        self.0.status == 429
    }

    /// Returns `true` for an HTTP 5xx status or the API's internal error code 500.
    pub fn is_server_error(&self) -> bool {
        self.0.status >= 500 || self.0.code == Some(ErrorCode::InternalServerError.code())
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(code) = self.0.code {
            write!(f, "code {code}")?;
            if let Some(key) = &self.0.key {
                write!(f, " ({key})")?;
            }
            write!(f, ": ")?;
        }
        write!(f, "{} (HTTP {})", self.0.message, self.0.status)
    }
}

/// The API error codes this SDK knows.
///
/// Compare with [`ApiError::error_code`]; the number itself is available from
/// [`ApiError::code`] even for codes not listed here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
#[repr(i64)]
pub enum ErrorCode {
    /// 1: invalid API token or signature.
    AuthError = 1,
    /// 2: invalid service or parameters.
    InvalidServiceOrParams = 2,
    /// 5: internal wallet not found; contact support.
    WalletNotFound = 5,
    /// 6: insufficient funds.
    InsufficientFunds = 6,
    /// 10: invalid TRON address, or the address already has an active
    /// subscription.
    InvalidTronAddress = 10,
    /// 11: invalid energy amount.
    InvalidEnergyAmount = 11,
    /// 12: invalid duration.
    InvalidDuration = 12,
    /// 20: transaction not found. The API reports it under the key
    /// `subscription_not_found`.
    TransactionNotFound = 20,
    /// 21: cannot stop subscription, for example because it has a transactions
    /// limit.
    CannotStopSubscription = 21,
    /// 24: address not activated; activate it first.
    AddressNotActivated = 24,
    /// 25: address already activated; no action needed.
    AddressAlreadyActivated = 25,
    /// 30: AML check not found.
    AmlCheckNotFound = 30,
    /// 35: service temporarily unavailable.
    ServiceNotAvailable = 35,
    /// 50: invalid bandwidth amount.
    InvalidBandwidthAmount = 50,
    /// 500: internal server error; contact support if it persists.
    InternalServerError = 500,
}

impl ErrorCode {
    /// Returns the number the API uses for this code.
    ///
    /// ```
    /// use tronzap_sdk::ErrorCode;
    ///
    /// assert_eq!(ErrorCode::InsufficientFunds.code(), 6);
    /// assert_eq!(ErrorCode::from_code(20), Some(ErrorCode::TransactionNotFound));
    /// assert_eq!(ErrorCode::from_code(7), None);
    /// ```
    pub fn code(self) -> i64 {
        self as i64
    }

    /// Returns the known code for a number.
    pub fn from_code(code: i64) -> Option<ErrorCode> {
        Some(match code {
            1 => ErrorCode::AuthError,
            2 => ErrorCode::InvalidServiceOrParams,
            5 => ErrorCode::WalletNotFound,
            6 => ErrorCode::InsufficientFunds,
            10 => ErrorCode::InvalidTronAddress,
            11 => ErrorCode::InvalidEnergyAmount,
            12 => ErrorCode::InvalidDuration,
            20 => ErrorCode::TransactionNotFound,
            21 => ErrorCode::CannotStopSubscription,
            24 => ErrorCode::AddressNotActivated,
            25 => ErrorCode::AddressAlreadyActivated,
            30 => ErrorCode::AmlCheckNotFound,
            35 => ErrorCode::ServiceNotAvailable,
            50 => ErrorCode::InvalidBandwidthAmount,
            500 => ErrorCode::InternalServerError,
            _ => return None,
        })
    }
}

/// Why no response arrived.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum TransportErrorKind {
    /// The request exceeded its timeout.
    Timeout,
    /// DNS resolution or the TCP connection failed.
    Connect,
    /// The TLS handshake or certificate verification failed.
    Tls,
    /// Any other transport failure.
    Other,
}

/// A request that produced no HTTP response.
#[derive(Debug)]
pub struct TransportError {
    kind: TransportErrorKind,
    source: reqwest::Error,
}

impl TransportError {
    pub(crate) fn new(source: reqwest::Error) -> Self {
        TransportError { kind: classify(&source), source }
    }

    /// Returns why the request failed.
    pub fn kind(&self) -> TransportErrorKind {
        self.kind
    }

    /// Returns `true` when the request timed out.
    pub fn is_timeout(&self) -> bool {
        self.kind == TransportErrorKind::Timeout
    }

    /// Returns `true` when the connection could not be established.
    pub fn is_connect(&self) -> bool {
        self.kind == TransportErrorKind::Connect
    }

    /// Returns `true` when TLS failed.
    pub fn is_tls(&self) -> bool {
        self.kind == TransportErrorKind::Tls
    }
}

impl fmt::Display for TransportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let what = match self.kind {
            TransportErrorKind::Timeout => "request timed out",
            TransportErrorKind::Connect => "connection failed",
            TransportErrorKind::Tls => "tls error",
            TransportErrorKind::Other => "transport error",
        };
        write!(f, "tronzap {what}: {}", self.source)?;
        let mut cause = self.source.source();
        while let Some(e) = cause {
            write!(f, ": {e}")?;
            cause = e.source();
        }
        Ok(())
    }
}

impl StdError for TransportError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(&self.source)
    }
}

// reqwest exposes no TLS predicate. The backend's own error type is the reliable
// signal; the text match covers a backend linked in another version. An InvalidData
// I/O error while connecting comes from the TLS layer, as TCP never reports it.
fn classify(error: &reqwest::Error) -> TransportErrorKind {
    if error.is_timeout() {
        return TransportErrorKind::Timeout;
    }
    let mut cause = error.source();
    while let Some(e) = cause {
        let io = e.downcast_ref::<std::io::Error>();
        let invalid_data = io.is_some_and(|io| io.kind() == std::io::ErrorKind::InvalidData);
        if is_tls_backend_error(e) || (invalid_data && error.is_connect()) || mentions_tls(e) {
            return TransportErrorKind::Tls;
        }
        // io::Error::source skips the wrapped error itself; get_ref returns it.
        cause = match io.and_then(std::io::Error::get_ref) {
            Some(inner) => Some(inner as &(dyn StdError + 'static)),
            None => e.source(),
        };
    }
    if error.is_connect() { TransportErrorKind::Connect } else { TransportErrorKind::Other }
}

#[allow(unused_variables)]
fn is_tls_backend_error(e: &(dyn StdError + 'static)) -> bool {
    #[cfg(feature = "rustls")]
    if e.is::<rustls::Error>() {
        return true;
    }
    #[cfg(feature = "native-tls")]
    if e.is::<native_tls::Error>() {
        return true;
    }
    false
}

fn mentions_tls(e: &dyn StdError) -> bool {
    let text = e.to_string().to_ascii_lowercase();
    ["certificate", "tls", "ssl", "handshake"].iter().any(|word| text.contains(word))
}

/// Arguments or configuration the SDK rejected before sending anything.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    field: &'static str,
    message: String,
}

impl ValidationError {
    pub(crate) fn new(field: &'static str, message: impl Into<String>) -> Self {
        ValidationError { field, message: message.into() }
    }

    /// Returns the name of the rejected field, such as `address`.
    pub fn field(&self) -> &str {
        self.field
    }

    /// Returns what is wrong with the field.
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid tronzap request: {} {}", self.field, self.message)
    }
}

impl StdError for ValidationError {}

/// A request body that could not be encoded, or a 2xx response that could not
/// be decoded.
#[derive(Debug)]
pub struct SerializationError {
    message: String,
    status: Option<u16>,
    body: Option<String>,
    source: Option<serde_json::Error>,
}

impl SerializationError {
    pub(crate) fn request(source: serde_json::Error) -> Self {
        SerializationError {
            message: "cannot encode request".to_owned(),
            status: None,
            body: None,
            source: Some(source),
        }
    }

    pub(crate) fn response(
        message: impl Into<String>,
        status: u16,
        body: String,
        source: Option<serde_json::Error>,
    ) -> Self {
        SerializationError { message: message.into(), status: Some(status), body: Some(body), source }
    }

    /// Returns what went wrong.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the HTTP status of the response, or `None` for a request that could
    /// not be encoded.
    pub fn status(&self) -> Option<u16> {
        self.status
    }

    /// Returns the raw response body, or `None` for a request that could not be
    /// encoded.
    pub fn body(&self) -> Option<&str> {
        self.body.as_deref()
    }
}

impl fmt::Display for SerializationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "tronzap serialization error: {}", self.message)?;
        if let Some(status) = self.status {
            write!(f, " (HTTP {status})")?;
        }
        if let Some(source) = &self.source {
            write!(f, ": {source}")?;
        }
        Ok(())
    }
}

impl StdError for SerializationError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source.as_ref().map(|e| e as _)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn api_error(code: Option<i64>, key: Option<&str>) -> ApiError {
        ApiError::new(ApiErrorData {
            status: 200,
            code,
            key: key.map(str::to_owned),
            message: "Insufficient funds".to_owned(),
            request_id: None,
            body: String::new(),
            retry_after: None,
        })
    }

    #[test]
    fn error_codes_round_trip() {
        for code in -1..=600 {
            if let Some(known) = ErrorCode::from_code(code) {
                assert_eq!(known.code(), code);
            }
        }
        assert_eq!(ErrorCode::from_code(1), Some(ErrorCode::AuthError));
        assert_eq!(ErrorCode::from_code(50), Some(ErrorCode::InvalidBandwidthAmount));
        assert_eq!(ErrorCode::from_code(0), None);
    }

    #[test]
    fn display_includes_code_key_and_status() {
        let e = TronzapError::Api(api_error(Some(6), Some("insufficient_funds")));
        assert_eq!(
            e.to_string(),
            "tronzap api error: code 6 (insufficient_funds): Insufficient funds (HTTP 200)"
        );
        let e = api_error(None, None);
        assert_eq!(e.to_string(), "Insufficient funds (HTTP 200)");
    }

    #[test]
    fn errors_are_send_sync_and_static() {
        fn assert_bounds<T: Send + Sync + 'static>() {}
        assert_bounds::<TronzapError>();
    }

    #[test]
    fn server_error_includes_code_500() {
        assert!(api_error(Some(500), None).is_server_error());
        assert!(!api_error(Some(6), None).is_server_error());
    }
}
