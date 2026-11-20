//! Bulk orders, against a local HTTP server.

#![allow(clippy::unwrap_used, clippy::panic)]
mod common;

use std::time::Duration;

use common::{api_error, assert_nothing_sent, client, ok, only_request, respond};
use serde_json::{Value, json};
use tronzap_sdk::models::{BulkRowStatus, BulkStatus};
use tronzap_sdk::requests::{
    BulkCalculateRequest, BulkCreateRequest, BulkHistoryRequest, BulkItem, BulkOrderRequest,
    BulkReportRequest,
};
use tronzap_sdk::{ErrorCode, TronzapError};
use wiremock::{MockServer, ResponseTemplate};

const CSV: &str = "row,address,amount,status,reason,charged,transaction_id\n\
1,TPY1Kb8cKAZQfm95gXQQs2Mh8Uygtos21D,65000,delivered,,3.25,01kb2m9c4de5fg6hj7km8np9qr\n\
2,TQn9Y2khEsLJW1ChVWFMSMeRDow5KcbLSE,131000,skipped,address_not_activated,,\n";

fn calculation() -> Value {
    json!({
        "id": "01kb2m7a1bc2de3fg4hj5km6np", "status": "draft", "resource": "energy", "duration": 1,
        "total_amount": 1240.1, "accepted": 2, "rejected_count": 1,
        "rejected": [{"row": 3, "address": "abc", "reason": "invalid_address"}],
        "expires_at": "2026-11-16T12:15:00+00:00"
    })
}

fn order() -> Value {
    json!({
        "id": "01kb2m5x9qj4v8n3c7d6e5f4g3", "external_id": "my-order-42", "status": "partial",
        "resource": "energy", "duration": 1, "force": true, "max_price_increase": 20,
        "webhook_url": "https://example.com/tronzap/bulk", "total": 4, "processed": 4, "delivered": 2,
        "failed": 0, "in_review": 0, "skipped": 2,
        "skipped_by_reason": {"address_not_activated": 1, "price_increased": 1}, "rejected_count": 1,
        "total_amount": "1240.10", "charged_amount": "620.05", "created_at": "2026-11-16T12:05:00+00:00",
        "started_at": "2026-11-16T12:06:00+00:00", "finalized_at": "2026-11-16T12:09:00+00:00"
    })
}

#[tokio::test]
async fn bulk_calculate() {
    let server = MockServer::start().await;
    respond(&server, "/v1/bulk/calculate", ok(calculation())).await;

    let request = BulkCalculateRequest::new([BulkItem::new("TAddr1", 65000), BulkItem::new("abc", 1)]);
    let calculation = client(&server).bulk_calculate(&request).await.unwrap();
    assert_eq!(
        only_request(&server).await,
        json!({"resource": "energy", "duration": 1, "items": [
            {"address": "TAddr1", "amount": 65000}, {"address": "abc", "amount": 1}
        ]})
    );
    assert_eq!(calculation.id, "01kb2m7a1bc2de3fg4hj5km6np");
    assert_eq!(calculation.status, BulkStatus::Draft);
    assert_eq!((calculation.accepted, calculation.rejected_count), (2, 1));
    assert_eq!(calculation.total_amount.to_string(), "1240.1");
    assert_eq!(calculation.rejected[0].row, 3);
    assert_eq!(calculation.rejected[0].reason, "invalid_address");
    assert_eq!(calculation.expires_at.unwrap().unix_timestamp(), Some(1_794_831_300));
}

#[tokio::test]
async fn bulk_calculate_without_valid_rows() {
    let server = MockServer::start().await;
    respond(
        &server,
        "/v1/bulk/calculate",
        ResponseTemplate::new(200).set_body_json(json!({
            "code": 94, "error": "No valid items", "key": "bulk_no_valid_items",
            "result": {"rejected": [{"row": 1, "address": "abc", "reason": "invalid_address"},
                                    {"row": 2, "address": "abc", "reason": "duplicate"}]}
        })),
    )
    .await;

    let request = BulkCalculateRequest::new([BulkItem::new("abc", 1), BulkItem::new("abc", 1)]);
    let e = api_error(client(&server).bulk_calculate(&request).await);
    assert_eq!(e.error_code(), Some(ErrorCode::BulkNoValidItems));
    assert_eq!(e.key(), Some("bulk_no_valid_items"));
    let reasons: Vec<_> = e.bulk_rejected().unwrap().iter().map(|r| r.reason.as_str()).collect();
    assert_eq!(reasons, ["invalid_address", "duplicate"]);
    assert_eq!(e.result().unwrap()["rejected"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn bulk_create() {
    let server = MockServer::start().await;
    respond(&server, "/v1/bulk/create", ok(order())).await;

    let request =
        BulkCreateRequest::new("calc-1").external_id("my-order-42").force(true).max_price_increase(0);
    let order = client(&server).bulk_create(&request).await.unwrap();
    assert_eq!(
        only_request(&server).await,
        json!({"id": "calc-1", "external_id": "my-order-42", "force": true, "max_price_increase": 0})
    );
    assert_eq!(order.status, BulkStatus::Partial);
    assert_eq!(order.external_id.as_deref(), Some("my-order-42"));
    assert_eq!((order.total, order.processed, order.delivered, order.skipped), (4, 4, 2, 2));
    assert_eq!(order.skipped_by_reason["price_increased"], 1);
    assert_eq!(order.charged_amount.to_string(), "620.05");
    assert!(order.force);
    assert_eq!(order.max_price_increase, 20);
    assert!(order.finalized_at.is_some());
}

#[tokio::test]
async fn bulk_create_when_the_price_changed() {
    let server = MockServer::start().await;
    let mut new_calculation = calculation();
    new_calculation["id"] = json!("calc-2");
    new_calculation["total_amount"] = json!("1300.5");
    respond(
        &server,
        "/v1/bulk/create",
        ResponseTemplate::new(200).set_body_json(json!({
            "code": 92, "error": "Price changed", "key": "bulk_price_changed", "result": new_calculation
        })),
    )
    .await;

    let e = api_error(client(&server).bulk_create(&BulkCreateRequest::new("calc-1")).await);
    assert_eq!(e.error_code(), Some(ErrorCode::BulkPriceChanged));
    let calculation = e.bulk_calculation().unwrap();
    assert_eq!((calculation.id.as_str(), calculation.total_amount.to_string()), ("calc-2", "1300.5".into()));
}

#[tokio::test]
async fn bulk_check_of_a_running_order() {
    let server = MockServer::start().await;
    let mut running = order();
    running["status"] = json!("processing");
    running["skipped_by_reason"] = json!([]);
    running["finalized_at"] = Value::Null;
    running["external_id"] = Value::Null;
    respond(&server, "/v1/bulk/check", ok(running)).await;

    let order = client(&server).bulk_check(&BulkOrderRequest::by_external_id("my-order-42")).await.unwrap();
    assert_eq!(only_request(&server).await, json!({"external_id": "my-order-42"}));
    assert_eq!(order.status, BulkStatus::Processing);
    assert!(order.skipped_by_reason.is_empty());
    assert_eq!((order.finalized_at, order.external_id), (None, None));
}

#[tokio::test]
async fn bulk_check_of_an_unknown_order() {
    let server = MockServer::start().await;
    respond(
        &server,
        "/v1/bulk/check",
        ResponseTemplate::new(404).set_body_json(json!({"code": 90, "error": "Bulk order not found"})),
    )
    .await;

    let e = api_error(client(&server).bulk_check(&BulkOrderRequest::by_id("missing")).await);
    assert_eq!((e.status(), e.error_code()), (404, Some(ErrorCode::BulkNotFound)));
    assert_eq!(e.result(), None);
}

#[tokio::test]
async fn get_bulk_history() {
    let server = MockServer::start().await;
    respond(
        &server,
        "/v1/bulk/history",
        ok(json!({"page": 2, "per_page": 5, "total": 6, "items": [order()]})),
    )
    .await;

    let history =
        client(&server).get_bulk_history(&BulkHistoryRequest::new().page(2).per_page(5)).await.unwrap();
    assert_eq!(only_request(&server).await, json!({"page": 2, "per_page": 5}));
    assert_eq!((history.page, history.per_page, history.total), (2, 5, 6));
    assert_eq!(history.items[0].id, "01kb2m5x9qj4v8n3c7d6e5f4g3");
}

#[tokio::test]
async fn bulk_report_returns_the_csv() {
    let server = MockServer::start().await;
    respond(
        &server,
        "/v1/bulk/report",
        ResponseTemplate::new(200).set_body_raw(CSV, "text/csv; charset=UTF-8"),
    )
    .await;

    let request = BulkReportRequest::by_id("bulk-1").status(BulkRowStatus::Skipped);
    assert_eq!(client(&server).bulk_report(&request).await.unwrap(), CSV);
    assert_eq!(only_request(&server).await, json!({"id": "bulk-1", "status": "skipped"}));
}

#[tokio::test]
async fn bulk_report_is_utf8_without_a_charset() {
    let server = MockServer::start().await;
    let text = "row,address\n1,адреса-😀\n";
    respond(&server, "/v1/bulk/report", ResponseTemplate::new(200).set_body_raw(text, "text/csv")).await;

    let report = client(&server).bulk_report(&BulkReportRequest::by_external_id("order-42")).await.unwrap();
    assert_eq!(report, text);
}

#[tokio::test]
async fn bulk_report_errors() {
    let server = MockServer::start().await;
    respond(
        &server,
        "/v1/bulk/report",
        ResponseTemplate::new(404).set_body_json(json!({"code": 90, "error": "Bulk order not found"})),
    )
    .await;
    let e = api_error(client(&server).bulk_report(&BulkReportRequest::by_id("missing")).await);
    assert_eq!(e.error_code(), Some(ErrorCode::BulkNotFound));

    let server = MockServer::start().await;
    respond(&server, "/v1/bulk/report", ResponseTemplate::new(502).set_body_raw("<html/>", "text/html"))
        .await;
    let e = api_error(client(&server).bulk_report(&BulkReportRequest::by_id("bulk-1")).await);
    assert!(e.is_server_error());

    for (body, content_type) in
        [("row,address\n", "text/plain"), (r#"{"code":0,"result":{}}"#, "application/json")]
    {
        let server = MockServer::start().await;
        respond(&server, "/v1/bulk/report", ResponseTemplate::new(200).set_body_raw(body, content_type))
            .await;
        match client(&server).bulk_report(&BulkReportRequest::by_id("bulk-1")).await {
            Err(TronzapError::Serialization(e)) => assert_eq!(e.body(), Some(body)),
            other => panic!("expected a serialization error, got {other:?}"),
        }
    }
}

#[tokio::test]
async fn rate_limit_reported_by_the_api() {
    let server = MockServer::start().await;
    respond(
        &server,
        "/v1/bulk/history",
        ResponseTemplate::new(429)
            .insert_header("Retry-After", "30")
            .set_body_json(json!({"code": 429, "error": "Too many requests", "key": "rate_limited"})),
    )
    .await;

    let e = api_error(client(&server).get_bulk_history(&BulkHistoryRequest::new()).await);
    assert_eq!(e.error_code(), Some(ErrorCode::RateLimited));
    assert_eq!(e.retry_after(), Some(Duration::from_secs(30)));
    assert!(e.is_rate_limited());
}

#[tokio::test]
async fn invalid_bulk_requests_are_not_sent() {
    let server = MockServer::start().await;
    let client = client(&server);
    assert!(matches!(
        client.bulk_calculate(&BulkCalculateRequest::new([])).await,
        Err(TronzapError::Validation(e)) if e.field() == "items"
    ));
    assert!(matches!(
        client.bulk_create(&BulkCreateRequest::new("")).await,
        Err(TronzapError::Validation(e)) if e.field() == "id"
    ));
    assert!(matches!(
        client.get_bulk_history(&BulkHistoryRequest::new().page(0)).await,
        Err(TronzapError::Validation(e)) if e.field() == "page"
    ));
    assert_nothing_sent(&server).await;
}
