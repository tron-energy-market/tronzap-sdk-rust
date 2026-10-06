#![allow(dead_code)]

use std::time::Duration;

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tronzap_sdk::{TronzapClient, TronzapError};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

pub const TOKEN: &str = "test-token";
pub const SECRET: &str = "test-secret";

pub fn client(server: &MockServer) -> TronzapClient {
    client_with_timeout(server, Duration::from_secs(5))
}

pub fn client_with_timeout(server: &MockServer, timeout: Duration) -> TronzapClient {
    TronzapClient::builder()
        .api_token(TOKEN)
        .api_secret(SECRET)
        .base_url(server.uri())
        .timeout(timeout)
        .build()
        .unwrap()
}

pub fn ok(result: Value) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(json!({"code": 0, "result": result}))
}

pub async fn respond(server: &MockServer, endpoint: &str, response: ResponseTemplate) {
    Mock::given(method("POST")).and(path(endpoint)).respond_with(response).mount(server).await;
}

/// Returns the JSON body of the only request the server received, after checking
/// the headers and that the signature matches the bytes that actually arrived.
pub async fn only_request(server: &MockServer) -> Value {
    let requests = server.received_requests().await.unwrap();
    assert_eq!(requests.len(), 1, "expected exactly one request");
    let request = &requests[0];

    let header = |name: &str| request.headers.get(name).and_then(|v| v.to_str().ok()).unwrap_or_default();
    assert_eq!(header("authorization"), format!("Bearer {TOKEN}"));
    assert_eq!(header("content-type"), "application/json");
    assert_eq!(header("accept"), "application/json");
    assert!(header("user-agent").starts_with("tronzap-sdk-rust/"), "{}", header("user-agent"));

    let mut hasher = Sha256::new();
    hasher.update(&request.body);
    hasher.update(SECRET.as_bytes());
    let expected: String = hasher.finalize().iter().map(|b| format!("{b:02x}")).collect();
    assert_eq!(header("x-signature"), expected, "signature must cover the received body");

    serde_json::from_slice(&request.body).unwrap()
}

pub async fn assert_nothing_sent(server: &MockServer) {
    assert!(server.received_requests().await.unwrap().is_empty(), "no request may be sent");
}

pub fn api_error(result: Result<impl std::fmt::Debug, TronzapError>) -> tronzap_sdk::ApiError {
    match result {
        Err(TronzapError::Api(e)) => e,
        other => panic!("expected TronzapError::Api, got {other:?}"),
    }
}
