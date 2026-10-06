//! Failure handling against a local HTTP server.

#![allow(clippy::unwrap_used, clippy::panic)]
mod common;

use std::time::Duration;

use common::{api_error, assert_nothing_sent, client, client_with_timeout, ok, respond};
use serde_json::json;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tronzap_sdk::models::AmlDirection;
use tronzap_sdk::requests::{
    AmlCheckRequest, AmlHistoryRequest, BandwidthTransactionRequest, CalculateRequest,
    CheckTransactionRequest, EnergyTransactionRequest, EstimateEnergyRequest,
};
use tronzap_sdk::{ErrorCode, TransportErrorKind, TronzapClient, TronzapError};
use wiremock::{MockServer, ResponseTemplate};

fn transport_kind(result: Result<impl std::fmt::Debug, TronzapError>) -> TransportErrorKind {
    match result {
        Err(TronzapError::Transport(e)) => e.kind(),
        other => panic!("expected TronzapError::Transport, got {other:?}"),
    }
}

fn plain_client(base_url: &str) -> TronzapClient {
    TronzapClient::builder()
        .api_token(common::TOKEN)
        .api_secret(common::SECRET)
        .base_url(base_url)
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap()
}

#[tokio::test]
async fn api_error_with_status_200() {
    let server = MockServer::start().await;
    let body =
        json!({"code": 6, "error": "Insufficient funds", "key": "insufficient_funds", "request_id": "req-1"});
    respond(&server, "/v1/transaction/new", ResponseTemplate::new(200).set_body_json(&body)).await;

    let request = EnergyTransactionRequest::new("TAddr", 65000);
    let e = api_error(client(&server).create_energy_transaction(&request).await);
    assert_eq!(e.status(), 200);
    assert_eq!(e.code(), Some(6));
    assert_eq!(e.error_code(), Some(ErrorCode::InsufficientFunds));
    assert_eq!(e.key(), Some("insufficient_funds"));
    assert_eq!(e.message(), "Insufficient funds");
    assert_eq!(e.request_id(), Some("req-1"));
    assert_eq!(serde_json::from_str::<serde_json::Value>(e.body()).unwrap(), body);
}

#[tokio::test]
async fn api_error_with_status_4xx() {
    let server = MockServer::start().await;
    respond(
        &server,
        "/v1/estimate-energy",
        ResponseTemplate::new(422).set_body_json(json!({
            "code": 10, "error": "Invalid TRON address", "key": "invalid_tron_address.from_address"
        })),
    )
    .await;

    let e = api_error(client(&server).estimate_energy(&EstimateEnergyRequest::new("bad", "TTo")).await);
    assert_eq!((e.status(), e.error_code()), (422, Some(ErrorCode::InvalidTronAddress)));
    assert_eq!(e.key(), Some("invalid_tron_address.from_address"));
}

#[tokio::test]
async fn unknown_api_code_keeps_the_number() {
    let server = MockServer::start().await;
    respond(
        &server,
        "/v1/balance",
        ResponseTemplate::new(200).set_body_json(json!({"code": 777, "error": "New"})),
    )
    .await;

    let e = api_error(client(&server).get_balance().await);
    assert_eq!((e.code(), e.error_code()), (Some(777), None));
}

#[tokio::test]
async fn transaction_not_found_uses_subscription_key() {
    let server = MockServer::start().await;
    respond(
        &server,
        "/v1/transaction/check",
        ResponseTemplate::new(404).set_body_json(json!({
            "code": 20, "error": "Subscription not found", "key": "subscription_not_found"
        })),
    )
    .await;

    let e = api_error(client(&server).check_transaction(&CheckTransactionRequest::by_id("nope")).await);
    assert_eq!(e.error_code(), Some(ErrorCode::TransactionNotFound));
}

#[tokio::test]
async fn address_already_activated() {
    let server = MockServer::start().await;
    respond(
        &server,
        "/v1/transaction/new",
        ResponseTemplate::new(200).set_body_json(json!({"code": 25, "error": "Address already activated"})),
    )
    .await;

    let request = tronzap_sdk::requests::AddressActivationRequest::new("TAddr");
    let e = api_error(client(&server).create_address_activation_transaction(&request).await);
    assert_eq!(e.error_code(), Some(ErrorCode::AddressAlreadyActivated));
}

#[tokio::test]
async fn auth_error_code() {
    let server = MockServer::start().await;
    respond(
        &server,
        "/v1/balance",
        ResponseTemplate::new(200).set_body_json(json!({"code": 1, "error": "Authentication error"})),
    )
    .await;

    match client(&server).get_balance().await {
        Err(TronzapError::Authentication(e)) => {
            assert_eq!(e.error_code(), Some(ErrorCode::AuthError));
            assert_eq!(e.message(), "Authentication error");
        }
        other => panic!("expected an authentication error, got {other:?}"),
    }
}

#[tokio::test]
async fn http_401_and_403_without_payload() {
    for status in [401, 403] {
        let server = MockServer::start().await;
        respond(&server, "/v1/balance", ResponseTemplate::new(status).set_body_string("Unauthorized")).await;

        match client(&server).get_balance().await {
            Err(TronzapError::Authentication(e)) => {
                assert_eq!((e.status(), e.code(), e.body()), (status, None, "Unauthorized"));
            }
            other => panic!("expected an authentication error for {status}, got {other:?}"),
        }
    }
}

#[tokio::test]
async fn http_429_with_retry_after() {
    let server = MockServer::start().await;
    respond(
        &server,
        "/v1/balance",
        ResponseTemplate::new(429).insert_header("Retry-After", "12").set_body_string("slow down"),
    )
    .await;

    let e = api_error(client(&server).get_balance().await);
    assert!(e.is_rate_limited());
    assert_eq!(e.retry_after(), Some(Duration::from_secs(12)));
    assert_eq!(e.message(), "too many requests");
}

#[tokio::test]
async fn http_5xx_and_other_statuses() {
    for (status, message) in [(500, "server error"), (503, "server error"), (418, "HTTP error 418")] {
        let server = MockServer::start().await;
        respond(&server, "/v1/balance", ResponseTemplate::new(status).set_body_string("<html>down</html>"))
            .await;

        let e = api_error(client(&server).get_balance().await);
        assert_eq!((e.status(), e.code(), e.message()), (status, None, message));
        assert_eq!(e.body(), "<html>down</html>");
        assert_eq!(e.is_server_error(), status >= 500);
    }
}

#[tokio::test]
async fn invalid_json_with_status_200() {
    let server = MockServer::start().await;
    respond(&server, "/v1/balance", ResponseTemplate::new(200).set_body_string("{\"code\":0,")).await;

    match client(&server).get_balance().await {
        Err(TronzapError::Serialization(e)) => {
            assert_eq!(e.message(), "invalid JSON response");
            assert_eq!(e.status(), Some(200));
            assert_eq!(e.body(), Some("{\"code\":0,"));
        }
        other => panic!("expected a serialization error, got {other:?}"),
    }
}

#[tokio::test]
async fn unusable_success_bodies() {
    let cases = [
        (json!([]), "response is not a JSON object"),
        (json!("text"), "response is not a JSON object"),
        (json!({"code": 0}), "missing result in response"),
        (json!({"code": 0, "result": null}), "missing result in response"),
        (json!({"code": 0, "result": {"balance": {"nested": true}}}), "unexpected result in response"),
    ];
    for (body, message) in cases {
        let server = MockServer::start().await;
        respond(&server, "/v1/balance", ResponseTemplate::new(200).set_body_json(&body)).await;

        match client(&server).get_balance().await {
            Err(TronzapError::Serialization(e)) => assert_eq!(e.message(), message, "{body}"),
            other => panic!("expected a serialization error for {body}, got {other:?}"),
        }
    }
}

#[tokio::test]
async fn missing_code_is_an_api_error() {
    let server = MockServer::start().await;
    respond(
        &server,
        "/v1/balance",
        ResponseTemplate::new(200).set_body_json(json!({"result": {"balance": 1}})),
    )
    .await;

    let e = api_error(client(&server).get_balance().await);
    assert_eq!(e.code(), None);
}

#[tokio::test]
async fn timeout() {
    let server = MockServer::start().await;
    respond(&server, "/v1/balance", ok(json!({"balance": 1})).set_delay(Duration::from_secs(2))).await;

    let result = client_with_timeout(&server, Duration::from_millis(100)).get_balance().await;
    assert!(matches!(&result, Err(e) if e.is_timeout()), "{result:?}");
    assert_eq!(transport_kind(result), TransportErrorKind::Timeout);
}

#[tokio::test]
async fn own_http_client_keeps_its_timeout() {
    let server = MockServer::start().await;
    respond(&server, "/v1/balance", ok(json!({"balance": 1})).set_delay(Duration::from_secs(2))).await;

    let http = tronzap_sdk::reqwest::Client::builder().timeout(Duration::from_millis(100)).build().unwrap();
    let client = TronzapClient::builder()
        .api_token(common::TOKEN)
        .api_secret(common::SECRET)
        .base_url(server.uri())
        .http_client(http)
        .build()
        .unwrap();
    assert_eq!(transport_kind(client.get_balance().await), TransportErrorKind::Timeout);
}

#[tokio::test]
async fn builder_timeout_replaces_own_http_client_timeout() {
    let server = MockServer::start().await;
    respond(&server, "/v1/balance", ok(json!({"balance": 1})).set_delay(Duration::from_millis(300))).await;

    let http = tronzap_sdk::reqwest::Client::builder().timeout(Duration::from_millis(100)).build().unwrap();
    let client = TronzapClient::builder()
        .api_token(common::TOKEN)
        .api_secret(common::SECRET)
        .base_url(server.uri())
        .http_client(http)
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();
    assert_eq!(client.get_balance().await.unwrap().balance.to_string(), "1");
}

#[tokio::test]
async fn dropping_the_future_cancels_without_error() {
    let server = MockServer::start().await;
    respond(&server, "/v1/balance", ok(json!({"balance": 1})).set_delay(Duration::from_secs(5))).await;

    let client = client(&server);
    let outcome = tokio::time::timeout(Duration::from_millis(100), client.get_balance()).await;
    assert!(outcome.is_err(), "the caller's own deadline fires first");
}

#[tokio::test]
async fn connection_refused() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    drop(listener);

    let result = plain_client(&format!("http://{address}")).get_balance().await;
    assert_eq!(transport_kind(result), TransportErrorKind::Connect);
}

#[tokio::test]
async fn connection_closed_without_response() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move {
        if let Ok((mut socket, _)) = listener.accept().await {
            let _ = socket.shutdown().await;
        }
    });

    let result = plain_client(&format!("http://{address}")).get_balance().await;
    assert!(matches!(result, Err(TronzapError::Transport(_))), "{result:?}");
}

#[cfg(any(feature = "rustls", feature = "native-tls"))]
#[tokio::test]
async fn untrusted_tls_certificate() {
    let generated = rcgen::generate_simple_self_signed(vec!["localhost".to_owned()]).unwrap();
    let cert = generated.cert.der().clone();
    let key =
        tokio_rustls::rustls::pki_types::PrivateKeyDer::Pkcs8(generated.signing_key.serialize_der().into());
    let config = tokio_rustls::rustls::ServerConfig::builder_with_provider(std::sync::Arc::new(
        tokio_rustls::rustls::crypto::aws_lc_rs::default_provider(),
    ))
    .with_safe_default_protocol_versions()
    .unwrap()
    .with_no_client_auth()
    .with_single_cert(vec![cert], key)
    .unwrap();
    let acceptor = tokio_rustls::TlsAcceptor::from(std::sync::Arc::new(config));

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        while let Ok((socket, _)) = listener.accept().await {
            let _ = acceptor.accept(socket).await;
        }
    });

    let result = plain_client(&format!("https://localhost:{port}")).get_balance().await;
    let message = format!("{result:?}");
    assert_eq!(transport_kind(result), TransportErrorKind::Tls, "{message}");
}

#[cfg(any(feature = "rustls", feature = "native-tls"))]
#[tokio::test]
async fn tls_to_a_plain_http_server() {
    let server = MockServer::start().await;
    let address = server.address();

    let result = plain_client(&format!("https://localhost:{}", address.port())).get_balance().await;
    let message = format!("{result:?}");
    assert_eq!(transport_kind(result), TransportErrorKind::Tls, "{message}");
}

#[tokio::test]
async fn invalid_requests_are_never_sent() {
    let server = MockServer::start().await;
    let client = client(&server);

    let failures = [
        client.get_address_info(" ").await.err(),
        client.check_aml_status("").await.err(),
        client.estimate_energy(&EstimateEnergyRequest::new("", "TTo")).await.err(),
        client.calculate(&CalculateRequest::new("TAddr", 0)).await.err(),
        client.create_energy_transaction(&EnergyTransactionRequest::new("TAddr", 0)).await.err(),
        client.create_bandwidth_transaction(&BandwidthTransactionRequest::new("", 345)).await.err(),
        client.check_transaction(&CheckTransactionRequest::by_external_id("")).await.err(),
        client
            .create_aml_check(&AmlCheckRequest::for_hash("BTC", "bc1q", " ", AmlDirection::Deposit))
            .await
            .err(),
        client.get_aml_history(&AmlHistoryRequest::new().per_page(0)).await.err(),
        client.request::<_, serde_json::Value>("  ", &json!({})).await.err(),
    ];
    for failure in failures {
        assert!(matches!(failure, Some(TronzapError::Validation(_))), "{failure:?}");
    }
    assert_nothing_sent(&server).await;
}

#[tokio::test]
async fn validation_error_names_the_field() {
    let server = MockServer::start().await;
    let request = EnergyTransactionRequest::new("TAddr", 65000).duration(0);
    match client(&server).create_energy_transaction(&request).await {
        Err(TronzapError::Validation(e)) => {
            assert_eq!(e.field(), "duration");
            assert_eq!(e.to_string(), "invalid tronzap request: duration must be positive");
        }
        other => panic!("expected a validation error, got {other:?}"),
    }
}

#[tokio::test]
async fn unserializable_params() {
    let server = MockServer::start().await;
    let mut params = std::collections::HashMap::new();
    params.insert((1, 2), "tuple keys cannot be JSON object keys");

    let result: Result<serde_json::Value, _> = client(&server).request("/v1/balance", &params).await;
    match result {
        Err(TronzapError::Serialization(e)) => assert_eq!((e.status(), e.body()), (None, None)),
        other => panic!("expected a serialization error, got {other:?}"),
    }
    assert_nothing_sent(&server).await;
}
