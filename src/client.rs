//! The API client.

use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, HeaderMap, RETRY_AFTER, USER_AGENT};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

use crate::config::{Config, TronzapClientBuilder};
use crate::error::{
    ApiError, ApiErrorData, Result, SerializationError, TransportError, TronzapError, ValidationError,
};
use crate::requests::{
    AddressActivationRequest, AddressParam, AmlCheckRequest, AmlHistoryRequest, BandwidthTransactionRequest,
    CalculateRequest, CheckTransactionRequest, EnergyTransactionRequest, EstimateEnergyRequest, IdParam,
    ResourceBundleTransactionRequest, required,
};
use crate::responses::{
    AddressInfo, AmlCheck, AmlHistory, AmlService, Balance, Calculation, DirectRechargeInfo, EnergyEstimate,
    Services, Transaction,
};

const MAX_RESPONSE_BYTES: usize = 8 << 20;

/// A client for the TronZap API.
///
/// Create one with [`TronzapClient::builder`] or [`TronzapClient::new`] and share
/// it: cloning is cheap, every clone uses the same connection pool, and the
/// client holds no mutable state, so it is safe to use from many tasks at once.
///
/// Every method is `async`. Dropping the returned future cancels the request.
///
/// ```no_run
/// use tronzap_sdk::TronzapClient;
/// use tronzap_sdk::requests::{EnergyTransactionRequest, EstimateEnergyRequest};
///
/// # async fn run() -> tronzap_sdk::Result<()> {
/// let client = TronzapClient::new("your_api_token", "your_api_secret")?;
///
/// let estimate = client
///     .estimate_energy(&EstimateEnergyRequest::new("TSenderAddress", "TRecipientAddress"))
///     .await?;
///
/// let request = EnergyTransactionRequest::new("TRecipientAddress", estimate.energy)
///     .external_id("order-42");
/// let tx = client.create_energy_transaction(&request).await?;
/// println!("transaction {} costs {} and is {}", tx.id, tx.amount, tx.status);
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct TronzapClient {
    config: Arc<Config>,
}

impl TronzapClient {
    /// Returns a builder to configure a client.
    pub fn builder() -> TronzapClientBuilder {
        TronzapClientBuilder::new()
    }

    /// Creates a client with the default settings.
    ///
    /// # Errors
    ///
    /// Same as [`TronzapClientBuilder::build`].
    pub fn new(api_token: impl Into<String>, api_secret: impl Into<String>) -> Result<Self> {
        Self::builder().api_token(api_token).api_secret(api_secret).build()
    }

    pub(crate) fn from_config(config: Config) -> Self {
        TronzapClient { config: Arc::new(config) }
    }

    /// Returns the API base URL the client sends requests to.
    pub fn base_url(&self) -> &str {
        &self.config.base_url
    }

    /// Returns the resources on sale and their prices.
    ///
    /// # Errors
    ///
    /// Every method returns [`TronzapError`] when the request fails; see its
    /// variants for the cases.
    pub async fn get_services(&self) -> Result<Services> {
        self.request("/v1/services", &Empty {}).await
    }

    /// Returns the account balance.
    ///
    /// # Errors
    ///
    /// See [`TronzapError`].
    pub async fn get_balance(&self) -> Result<Balance> {
        self.request("/v1/balance", &Empty {}).await
    }

    /// Returns the energy, bandwidth and token balances of a TRON address.
    ///
    /// # Errors
    ///
    /// [`TronzapError::Validation`] when `address` is blank; otherwise see
    /// [`TronzapError`].
    pub async fn get_address_info(&self, address: &str) -> Result<AddressInfo> {
        required("address", address).map_err(TronzapError::Validation)?;
        self.request("/v1/address-info", &AddressParam { address }).await
    }

    /// Estimates the energy a token transfer needs, and its cost.
    ///
    /// # Errors
    ///
    /// [`TronzapError::Validation`] for an invalid request; otherwise see
    /// [`TronzapError`].
    pub async fn estimate_energy(&self, request: &EstimateEnergyRequest) -> Result<EnergyEstimate> {
        validate(request.validate())?;
        self.request("/v1/estimate-energy", &request.wire()).await
    }

    /// Prices an energy purchase without creating a transaction.
    ///
    /// # Errors
    ///
    /// [`TronzapError::Validation`] for an invalid request; otherwise see
    /// [`TronzapError`].
    pub async fn calculate(&self, request: &CalculateRequest) -> Result<Calculation> {
        validate(request.validate())?;
        self.request("/v1/calculate", &request.wire()).await
    }

    /// Buys energy for an address. This debits the account balance.
    ///
    /// # Errors
    ///
    /// [`TronzapError::Validation`] for an invalid request; otherwise see
    /// [`TronzapError`].
    pub async fn create_energy_transaction(&self, request: &EnergyTransactionRequest) -> Result<Transaction> {
        validate(request.validate())?;
        self.request(TRANSACTION_NEW, &request.wire()).await
    }

    /// Buys bandwidth for an address. This debits the account balance.
    ///
    /// # Errors
    ///
    /// [`TronzapError::Validation`] for an invalid request; otherwise see
    /// [`TronzapError`].
    pub async fn create_bandwidth_transaction(
        &self,
        request: &BandwidthTransactionRequest,
    ) -> Result<Transaction> {
        validate(request.validate())?;
        self.request(TRANSACTION_NEW, &request.wire()).await
    }

    /// Buys energy and bandwidth in one transaction. This debits the account
    /// balance.
    ///
    /// # Errors
    ///
    /// [`TronzapError::Validation`] for an invalid request; otherwise see
    /// [`TronzapError`].
    pub async fn create_resource_bundle_transaction(
        &self,
        request: &ResourceBundleTransactionRequest,
    ) -> Result<Transaction> {
        validate(request.validate())?;
        self.request(TRANSACTION_NEW, &request.wire()).await
    }

    /// Activates a TRON address. This debits the account balance.
    ///
    /// An address that is already active fails with
    /// [`ErrorCode::AddressAlreadyActivated`](crate::ErrorCode::AddressAlreadyActivated).
    ///
    /// # Errors
    ///
    /// [`TronzapError::Validation`] for an invalid request; otherwise see
    /// [`TronzapError`].
    pub async fn create_address_activation_transaction(
        &self,
        request: &AddressActivationRequest,
    ) -> Result<Transaction> {
        validate(request.validate())?;
        self.request(TRANSACTION_NEW, &request.wire()).await
    }

    /// Returns the current state of a transaction.
    ///
    /// # Errors
    ///
    /// [`TronzapError::Validation`] for an invalid request; otherwise see
    /// [`TronzapError`].
    pub async fn check_transaction(&self, request: &CheckTransactionRequest) -> Result<Transaction> {
        validate(request.validate())?;
        self.request("/v1/transaction/check", &request.wire()).await
    }

    /// Returns the direct recharge address and rates.
    ///
    /// # Errors
    ///
    /// See [`TronzapError`].
    pub async fn get_direct_recharge_info(&self) -> Result<DirectRechargeInfo> {
        self.request("/v1/direct-recharge-info", &Empty {}).await
    }

    /// Returns the AML screening products and their prices.
    ///
    /// # Errors
    ///
    /// See [`TronzapError`].
    pub async fn get_aml_services(&self) -> Result<Vec<AmlService>> {
        let services: crate::responses::AmlServices = self.request("/v1/aml-checks", &Empty {}).await?;
        Ok(services.0)
    }

    /// Starts an AML screening of an address or a transaction. This debits the
    /// account balance.
    ///
    /// Screening runs in the background: poll [`check_aml_status`](Self::check_aml_status)
    /// until the status is [`AmlStatus::Completed`](crate::models::AmlStatus::Completed).
    ///
    /// # Errors
    ///
    /// [`TronzapError::Validation`] for an invalid request; otherwise see
    /// [`TronzapError`].
    pub async fn create_aml_check(&self, request: &AmlCheckRequest) -> Result<AmlCheck> {
        validate(request.validate())?;
        self.request("/v1/aml-checks/new", &request.wire()).await
    }

    /// Returns the state and, once completed, the result of an AML check.
    ///
    /// # Errors
    ///
    /// [`TronzapError::Validation`] when `id` is blank; otherwise see
    /// [`TronzapError`].
    pub async fn check_aml_status(&self, id: &str) -> Result<AmlCheck> {
        required("id", id).map_err(TronzapError::Validation)?;
        self.request("/v1/aml-checks/check", &IdParam { id }).await
    }

    /// Returns one page of past AML checks.
    ///
    /// # Errors
    ///
    /// [`TronzapError::Validation`] for an invalid request; otherwise see
    /// [`TronzapError`].
    pub async fn get_aml_history(&self, request: &AmlHistoryRequest) -> Result<AmlHistory> {
        validate(request.validate())?;
        self.request("/v1/aml-checks/history", &request.wire()).await
    }

    /// Sends a signed request to any endpoint and decodes its `result`.
    ///
    /// Use it for endpoints this SDK does not wrap yet, and prefer the typed
    /// methods otherwise. `endpoint` is a path such as `/v1/balance`.
    ///
    /// ```no_run
    /// # async fn run(client: tronzap_sdk::TronzapClient) -> tronzap_sdk::Result<()> {
    /// let balance: serde_json::Value = client.request("/v1/balance", &serde_json::json!({})).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// [`TronzapError::Validation`] when `endpoint` is blank,
    /// [`TronzapError::Serialization`] when `params` cannot be encoded as JSON;
    /// otherwise see [`TronzapError`].
    pub async fn request<P, R>(&self, endpoint: &str, params: &P) -> Result<R>
    where
        P: Serialize + ?Sized,
        R: DeserializeOwned,
    {
        let endpoint = endpoint.trim();
        required("endpoint", endpoint).map_err(TronzapError::Validation)?;
        let body = serde_json::to_vec(params)
            .map_err(|e| TronzapError::Serialization(SerializationError::request(e)))?;

        let config = &*self.config;
        let separator = if endpoint.starts_with('/') { "" } else { "/" };
        let url = format!("{}{separator}{endpoint}", config.base_url);

        let mut builder = config.http.post(url);
        if let Some(timeout) = config.timeout {
            builder = builder.timeout(timeout);
        }
        let response = builder
            .header(AUTHORIZATION, config.authorization.clone())
            .header("X-Signature", sign(&body, &config.api_secret))
            .header(CONTENT_TYPE, "application/json")
            .header(ACCEPT, "application/json")
            .header(USER_AGENT, config.user_agent.clone())
            .body(body)
            .send()
            .await
            .map_err(transport)?;

        let status = response.status().as_u16();
        let retry_after = retry_after(response.headers());
        let body = read_body(response).await?;
        decode(status, retry_after, &body)
    }
}

impl fmt::Debug for TronzapClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TronzapClient")
            .field("base_url", &self.config.base_url)
            .field("timeout", &self.config.timeout)
            .field("user_agent", &self.config.user_agent)
            .field("api_token", &"<redacted>")
            .field("api_secret", &"<redacted>")
            .finish()
    }
}

const TRANSACTION_NEW: &str = "/v1/transaction/new";

#[derive(Serialize)]
struct Empty {}

fn validate(result: std::result::Result<(), ValidationError>) -> Result<()> {
    result.map_err(TronzapError::Validation)
}

fn transport(error: reqwest::Error) -> TronzapError {
    TronzapError::Transport(TransportError::new(error))
}

fn sign(body: &[u8], secret: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(body);
    hasher.update(secret.as_bytes());
    hex(&hasher.finalize())
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(char::from(DIGITS[usize::from(b >> 4)]));
        out.push(char::from(DIGITS[usize::from(b & 0x0f)]));
    }
    out
}

fn retry_after(headers: &HeaderMap) -> Option<Duration> {
    let value = headers.get(RETRY_AFTER)?.to_str().ok()?;
    value.trim().parse().ok().map(Duration::from_secs)
}

async fn read_body(mut response: reqwest::Response) -> Result<Vec<u8>> {
    let status = response.status().as_u16();
    let mut body = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(transport)? {
        if body.len() + chunk.len() > MAX_RESPONSE_BYTES {
            return Err(TronzapError::Serialization(SerializationError::response(
                format!("response body exceeds {MAX_RESPONSE_BYTES} bytes"),
                status,
                String::new(),
                None,
            )));
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

// The API reports some failures with a 2xx status and others with 4xx/5xx, so a
// non-zero `code` is checked before the HTTP status.
fn decode<R: DeserializeOwned>(status: u16, retry_after: Option<Duration>, body: &[u8]) -> Result<R> {
    let success = (200..300).contains(&status);
    let text = || String::from_utf8_lossy(body).into_owned();

    let mut map = match serde_json::from_slice::<Value>(body) {
        Ok(Value::Object(map)) => map,
        _ if !success => {
            return Err(api_error(ApiErrorData {
                status,
                code: None,
                key: None,
                message: status_message(status),
                request_id: None,
                body: text(),
                retry_after,
            }));
        }
        Ok(_) => {
            return Err(TronzapError::Serialization(SerializationError::response(
                "response is not a JSON object",
                status,
                text(),
                None,
            )));
        }
        Err(e) => {
            return Err(TronzapError::Serialization(SerializationError::response(
                "invalid JSON response",
                status,
                text(),
                Some(e),
            )));
        }
    };

    let code = map.get("code").and_then(integer);
    if code != Some(0) || !success {
        let code = code.filter(|&c| c != 0);
        let message = string(&map, "error")
            .or_else(|| string(&map, "message"))
            .unwrap_or_else(|| if success { "unknown API error".to_owned() } else { status_message(status) });
        return Err(api_error(ApiErrorData {
            status,
            code,
            key: string(&map, "key"),
            message,
            request_id: string(&map, "request_id"),
            body: text(),
            retry_after,
        }));
    }

    match map.remove("result") {
        None | Some(Value::Null) => Err(TronzapError::Serialization(SerializationError::response(
            "missing result in response",
            status,
            text(),
            None,
        ))),
        Some(result) => R::deserialize(result).map_err(|e| {
            TronzapError::Serialization(SerializationError::response(
                "unexpected result in response",
                status,
                text(),
                Some(e),
            ))
        }),
    }
}

fn api_error(data: ApiErrorData) -> TronzapError {
    let auth_code = data.code == Some(crate::ErrorCode::AuthError.code());
    let auth_status = data.code.is_none() && matches!(data.status, 401 | 403);
    let error = ApiError::new(data);
    if auth_code || auth_status { TronzapError::Authentication(error) } else { TronzapError::Api(error) }
}

fn status_message(status: u16) -> String {
    match status {
        401 | 403 => "unauthorized".to_owned(),
        429 => "too many requests".to_owned(),
        500.. => "server error".to_owned(),
        _ => format!("HTTP error {status}"),
    }
}

fn integer(value: &Value) -> Option<i64> {
    match value {
        Value::Number(n) => n.as_i64(),
        Value::String(s) => s.trim().parse().ok(),
        _ => None,
    }
}

fn string(map: &Map<String, Value>, key: &str) -> Option<String> {
    match map.get(key)? {
        Value::String(s) if !s.is_empty() => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn run(status: u16, body: &str) -> Result<Value> {
        decode(status, None, body.as_bytes())
    }

    fn api(result: Result<Value>) -> ApiError {
        match result {
            Err(TronzapError::Api(e)) => e,
            other => panic!("expected an API error, got {other:?}"),
        }
    }

    fn serialization(result: Result<Value>) -> SerializationError {
        match result {
            Err(TronzapError::Serialization(e)) => e,
            other => panic!("expected a serialization error, got {other:?}"),
        }
    }

    #[test]
    fn signature_is_sha256_of_body_and_secret() {
        assert_eq!(sign(b"{}", "secret"), "6826e169e270a699d36407a3e3f9c9bba4778bb9b27c99e99a0bf47762093d46");
        assert_eq!(hex(&[0x00, 0x0f, 0xa5, 0xff]), "000fa5ff");
        assert_eq!(sign(b"", "").len(), 64);
        assert_eq!(sign(b"", ""), "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
    }

    #[test]
    fn success_returns_result() {
        assert_eq!(run(200, r#"{"code":0,"result":{"a":1}}"#).unwrap(), json!({"a": 1}));
        assert_eq!(run(200, r#"{"code":"0","result":[]}"#).unwrap(), json!([]));
    }

    #[test]
    fn api_code_wins_over_status() {
        let e = api(run(
            200,
            r#"{"code":6,"error":"Insufficient funds","key":"insufficient_funds","request_id":"r1"}"#,
        ));
        assert_eq!(
            (e.status(), e.code(), e.key(), e.request_id()),
            (200, Some(6), Some("insufficient_funds"), Some("r1"))
        );
        assert_eq!(e.message(), "Insufficient funds");
        let e = api(run(422, r#"{"code":10,"error":"Invalid TRON address"}"#));
        assert_eq!((e.status(), e.code()), (422, Some(10)));
        assert!(matches!(
            run(401, r#"{"code":1,"error":"Bad signature"}"#),
            Err(TronzapError::Authentication(_))
        ));
        assert!(matches!(run(200, r#"{"code":1}"#), Err(TronzapError::Authentication(_))));
    }

    #[test]
    fn missing_code_is_an_api_error() {
        let e = api(run(200, r#"{"result":{}}"#));
        assert_eq!((e.code(), e.message()), (None, "unknown API error"));
    }

    #[test]
    fn http_errors_without_payload() {
        let e = api(run(500, "<html>oops</html>"));
        assert_eq!(
            (e.status(), e.code(), e.message(), e.body()),
            (500, None, "server error", "<html>oops</html>")
        );
        assert!(e.is_server_error());
        let e = api(run(429, ""));
        assert!(e.is_rate_limited());
        assert_eq!(api(run(418, "[]")).message(), "HTTP error 418");
        let e = api(run(502, r#"{"code":0,"result":{}}"#));
        assert_eq!((e.status(), e.code()), (502, None));
        assert!(matches!(run(403, "Forbidden"), Err(TronzapError::Authentication(_))));
        let e = api(run(404, r#"{"message":"Not Found"}"#));
        assert_eq!(e.message(), "Not Found");
    }

    #[test]
    fn unreadable_success_responses() {
        assert_eq!(serialization(run(200, "{not json")).message(), "invalid JSON response");
        assert_eq!(serialization(run(200, r#""text""#)).message(), "response is not a JSON object");
        assert_eq!(serialization(run(200, "[]")).message(), "response is not a JSON object");
        assert_eq!(serialization(run(200, r#"{"code":0}"#)).message(), "missing result in response");
        assert_eq!(
            serialization(run(200, r#"{"code":0,"result":null}"#)).message(),
            "missing result in response"
        );
        let e = serialization(
            decode::<Balance>(200, None, br#"{"code":0,"result":{"balance":"lots"}}"#).map(|_| json!(null)),
        );
        assert_eq!(e.message(), "unexpected result in response");
        assert_eq!(e.status(), Some(200));
        assert!(std::error::Error::source(&e).is_some());
    }

    #[test]
    fn parses_retry_after_seconds() {
        let mut headers = HeaderMap::new();
        assert_eq!(retry_after(&headers), None);
        headers.insert(RETRY_AFTER, "7".parse().unwrap());
        assert_eq!(retry_after(&headers), Some(Duration::from_secs(7)));
        headers.insert(RETRY_AFTER, "Wed, 21 Oct 2026 07:28:00 GMT".parse().unwrap());
        assert_eq!(retry_after(&headers), None);
    }
}
