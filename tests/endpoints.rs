//! Request and response format of every endpoint, against a local HTTP server.

#![allow(clippy::unwrap_used, clippy::panic)]
mod common;

use common::{client, ok, only_request, respond};
use serde_json::json;
use tronzap_sdk::TronzapError;
use tronzap_sdk::models::{
    AmlCheckType, AmlDirection, AmlRiskLevel, AmlStatus, Service, SubscriptionStatus, TransactionStatus,
};
use tronzap_sdk::requests::{
    AddressActivationRequest, AmlCheckRequest, AmlHistoryRequest, BandwidthTransactionRequest,
    CalculateRequest, CheckTransactionRequest, EnergyTransactionRequest, EstimateEnergyRequest,
    ResourceBundleTransactionRequest, StartSubscriptionRequest, SubscriptionHistoryRequest,
    SubscriptionRequest, USDT_CONTRACT_ADDRESS,
};
use wiremock::{MockServer, ResponseTemplate};

const ADDRESS: &str = "TQrY8tryqsYVCYS3MFbtffiPp2ccyn4STm";

fn transaction(service: &str, params: serde_json::Value) -> serde_json::Value {
    json!({
        "id": "a1b2c3",
        "external_id": "order-42",
        "service": service,
        "params": params,
        "status": "pending",
        "amount": 5.47,
        "created_at": "2026-08-14T09:30:00+00:00",
        "hash": null
    })
}

#[tokio::test]
async fn get_services() {
    let server = MockServer::start().await;
    respond(
        &server,
        "/v1/services",
        ok(json!({
            "energy": [{"duration": 1, "min_amount": 65000, "max_amount": 10000000, "price": "0.03",
                        "price_32k": 0.96, "price_65k": 1.95, "price_131k": 3.93}],
            "bandwidth": [{"duration": 1, "min_amount": 300, "max_amount": 100000, "price": 1}],
            "activate_address": {"price": 1.4}
        })),
    )
    .await;

    let services = client(&server).get_services().await.unwrap();
    assert_eq!(only_request(&server).await, json!({}));
    assert_eq!(services.energy.len(), 1);
    assert_eq!(services.energy[0].min_amount, 65000);
    assert_eq!(services.energy[0].price.to_string(), "0.03");
    assert_eq!(services.energy[0].price_65k.to_string(), "1.95");
    assert_eq!(services.bandwidth[0].min_amount, 300);
    assert_eq!(services.activate_address.unwrap().price.to_string(), "1.4");
}

#[tokio::test]
async fn get_balance() {
    let server = MockServer::start().await;
    respond(&server, "/v1/balance", ok(json!({"balance": "125.38", "address": ADDRESS}))).await;

    let balance = client(&server).get_balance().await.unwrap();
    assert_eq!(only_request(&server).await, json!({}));
    assert_eq!(balance.balance.to_string(), "125.38");
    assert_eq!(balance.address, ADDRESS);
}

#[tokio::test]
async fn get_address_info() {
    let server = MockServer::start().await;
    respond(
        &server,
        "/v1/address-info",
        ok(json!({"resources": {"energy": 65000, "bandwidth": 345}, "balances": {"TRX": "12.5", "USDT": 100}})),
    )
    .await;

    let info = client(&server).get_address_info(ADDRESS).await.unwrap();
    assert_eq!(only_request(&server).await, json!({"address": ADDRESS}));
    assert_eq!((info.resources.energy, info.resources.bandwidth), (65000, 345));
    assert_eq!(info.balances["TRX"].to_string(), "12.5");
    assert_eq!(info.balances["USDT"].to_string(), "100");
}

#[tokio::test]
async fn estimate_energy() {
    let server = MockServer::start().await;
    respond(
        &server,
        "/v1/estimate-energy",
        ok(json!({"amount": 65000, "duration": 1, "price": 5.47, "activation_fee": 0,
                  "total": 5.47, "from_address": "TFrom", "to_address": ADDRESS,
                  "contract_address": USDT_CONTRACT_ADDRESS})),
    )
    .await;

    let request = EstimateEnergyRequest::new("TFrom", ADDRESS);
    let estimate = client(&server).estimate_energy(&request).await.unwrap();
    assert_eq!(only_request(&server).await, json!({"from_address": "TFrom", "to_address": ADDRESS}));
    assert_eq!(estimate.amount, 65000);
    assert_eq!(estimate.total.to_string(), "5.47");
    assert_eq!(estimate.contract_address, USDT_CONTRACT_ADDRESS);
}

#[tokio::test]
async fn estimate_energy_for_another_contract() {
    let server = MockServer::start().await;
    respond(&server, "/v1/estimate-energy", ok(json!({}))).await;

    let request = EstimateEnergyRequest::new("TFrom", ADDRESS).contract_address("TContract");
    client(&server).estimate_energy(&request).await.unwrap();
    assert_eq!(
        only_request(&server).await,
        json!({"from_address": "TFrom", "to_address": ADDRESS, "contract_address": "TContract"})
    );
}

#[tokio::test]
async fn calculate() {
    let server = MockServer::start().await;
    respond(
        &server,
        "/v1/calculate",
        ok(json!({"address": ADDRESS, "type": "energy", "amount": 65000, "duration": 24,
                  "price": "5.47", "activation_fee": "1.4", "total": "6.87"})),
    )
    .await;

    let request = CalculateRequest::new(ADDRESS, 65000).duration(24);
    let calculation = client(&server).calculate(&request).await.unwrap();
    assert_eq!(only_request(&server).await, json!({"address": ADDRESS, "amount": 65000, "duration": 24}));
    assert_eq!(calculation.service, Service::Energy);
    assert_eq!(calculation.amount, 65000);
    assert_eq!(calculation.duration, 24);
    assert_eq!(calculation.total.to_string(), "6.87");
}

#[tokio::test]
async fn create_energy_transaction() {
    let server = MockServer::start().await;
    respond(
        &server,
        "/v1/transaction/new",
        ok(transaction(
            "energy",
            json!({"address": ADDRESS, "amounts": {"energy": 65000}, "duration": 1, "activate_address": true}),
        )),
    )
    .await;

    let request =
        EnergyTransactionRequest::new(ADDRESS, 65000).external_id("order-42").activate_address(true);
    let tx = client(&server).create_energy_transaction(&request).await.unwrap();
    assert_eq!(
        only_request(&server).await,
        json!({
            "service": "energy",
            "params": {"address": ADDRESS, "amounts": {"energy": 65000}, "duration": 1, "activate_address": true},
            "external_id": "order-42"
        })
    );
    assert_eq!(tx.id, "a1b2c3");
    assert_eq!(tx.external_id.as_deref(), Some("order-42"));
    assert_eq!(tx.status, TransactionStatus::Pending);
    assert_eq!(tx.amount.to_string(), "5.47");
    assert_eq!(tx.params.amounts.energy, 65000);
    assert!(tx.params.activate_address);
    assert_eq!(tx.created_at.unwrap().unix_timestamp(), Some(1_786_699_800));
    assert_eq!(tx.hash, None);
}

#[tokio::test]
async fn create_bandwidth_transaction() {
    let server = MockServer::start().await;
    respond(
        &server,
        "/v1/transaction/new",
        ok(transaction(
            "bandwidth",
            json!({"address": ADDRESS, "amounts": {"bandwidth": 345}, "duration": 1}),
        )),
    )
    .await;

    let request = BandwidthTransactionRequest::new(ADDRESS, 345);
    let tx = client(&server).create_bandwidth_transaction(&request).await.unwrap();
    assert_eq!(
        only_request(&server).await,
        json!({"service": "bandwidth", "params": {"address": ADDRESS, "amounts": {"bandwidth": 345}, "duration": 1}})
    );
    assert_eq!(tx.service, Service::Bandwidth);
    assert_eq!(tx.params.amounts.bandwidth, 345);
}

#[tokio::test]
async fn create_resource_bundle_transaction() {
    let server = MockServer::start().await;
    respond(
        &server,
        "/v1/transaction/new",
        ok(transaction(
            "energy",
            json!({"address": ADDRESS, "amounts": {"energy": 65000, "bandwidth": 345}, "duration": 1}),
        )),
    )
    .await;

    let request = ResourceBundleTransactionRequest::new(ADDRESS, 65000, 345).external_id("bundle-1");
    let tx = client(&server).create_resource_bundle_transaction(&request).await.unwrap();
    assert_eq!(
        only_request(&server).await,
        json!({
            "service": "resource_bundle",
            "params": {"address": ADDRESS, "amounts": {"energy": 65000, "bandwidth": 345}, "duration": 1},
            "external_id": "bundle-1"
        })
    );
    assert_eq!((tx.params.amounts.energy, tx.params.amounts.bandwidth), (65000, 345));
}

#[tokio::test]
async fn create_address_activation_transaction() {
    let server = MockServer::start().await;
    respond(&server, "/v1/transaction/new", ok(transaction("activate_address", json!({"address": ADDRESS}))))
        .await;

    let request = AddressActivationRequest::new(ADDRESS);
    let tx = client(&server).create_address_activation_transaction(&request).await.unwrap();
    assert_eq!(
        only_request(&server).await,
        json!({"service": "activate_address", "params": {"address": ADDRESS}})
    );
    assert_eq!(tx.service, Service::ActivateAddress);
}

#[tokio::test]
async fn check_transaction_with_string_amount() {
    let server = MockServer::start().await;
    let mut result = transaction("energy", json!({"address": ADDRESS, "amounts": {"energy": 65000}}));
    result["amount"] = json!("5.47");
    result["status"] = json!("success");
    result["hash"] = json!("f00dfeed");
    respond(&server, "/v1/transaction/check", ok(result)).await;

    let request = CheckTransactionRequest::by_external_id("order-42");
    let tx = client(&server).check_transaction(&request).await.unwrap();
    assert_eq!(only_request(&server).await, json!({"external_id": "order-42"}));
    assert_eq!(tx.amount.to_string(), "5.47");
    assert_eq!(tx.status, TransactionStatus::Success);
    assert_eq!(tx.hash.as_deref(), Some("f00dfeed"));
}

#[tokio::test]
async fn check_transaction_by_both_ids() {
    let server = MockServer::start().await;
    respond(&server, "/v1/transaction/check", ok(json!({"id": "a1b2c3"}))).await;

    let request = CheckTransactionRequest::by_id("a1b2c3").external_id("order-42");
    client(&server).check_transaction(&request).await.unwrap();
    assert_eq!(only_request(&server).await, json!({"id": "a1b2c3", "external_id": "order-42"}));
}

#[tokio::test]
async fn get_direct_recharge_info() {
    let server = MockServer::start().await;
    respond(
        &server,
        "/v1/direct-recharge-info",
        ok(json!({"address": ADDRESS, "rates": [{"duration": 1, "min_energy": 32000, "max_energy": 3000000,
                  "price": 0.0841, "price_32k": 2.69, "price_65k": 5.47, "price_131k": 11.02}]})),
    )
    .await;

    let info = client(&server).get_direct_recharge_info().await.unwrap();
    assert_eq!(only_request(&server).await, json!({}));
    assert_eq!(info.address, ADDRESS);
    assert_eq!(info.rates[0].max_energy, 3_000_000);
}

#[tokio::test]
async fn get_aml_services() {
    let server = MockServer::start().await;
    respond(
        &server,
        "/v1/aml-checks",
        ok(json!([{"id": "address", "type": "address", "price": 1.5}, {"id": "hash", "type": "hash", "price": "2"}])),
    )
    .await;

    let services = client(&server).get_aml_services().await.unwrap();
    assert_eq!(only_request(&server).await, json!({}));
    assert_eq!(services.len(), 2);
    assert_eq!(services[1].check_type, AmlCheckType::Hash);
    assert_eq!(services[1].price.to_string(), "2");
}

#[tokio::test]
async fn get_aml_services_empty_as_object() {
    let server = MockServer::start().await;
    respond(&server, "/v1/aml-checks", ok(json!({}))).await;
    assert!(client(&server).get_aml_services().await.unwrap().is_empty());
}

#[tokio::test]
async fn create_aml_check() {
    let server = MockServer::start().await;
    respond(
        &server,
        "/v1/aml-checks/new",
        ok(json!({"id": "aml-1", "type": "hash", "address": "bc1q", "hash": "HASH", "direction": "withdrawal",
                  "network": "BTC", "status": "pending", "risk_score": null, "risk_level": null,
                  "blacklist": false, "risk_factors": [], "checked_at": null})),
    )
    .await;

    let request = AmlCheckRequest::for_hash("BTC", "bc1q", "HASH", AmlDirection::Withdrawal);
    let check = client(&server).create_aml_check(&request).await.unwrap();
    assert_eq!(
        only_request(&server).await,
        json!({"type": "hash", "network": "BTC", "address": "bc1q", "hash": "HASH", "direction": "withdrawal"})
    );
    assert_eq!(check.status, AmlStatus::Pending);
    assert_eq!(check.direction, Some(AmlDirection::Withdrawal));
    assert_eq!(check.risk_score, None);
    assert_eq!(check.checked_at, None);
}

#[tokio::test]
async fn check_aml_status() {
    let server = MockServer::start().await;
    respond(
        &server,
        "/v1/aml-checks/check",
        ok(json!({"id": "aml-1", "type": "address", "address": ADDRESS, "network": "TRX", "status": "completed",
                  "risk_score": "35.3", "risk_level": "medium", "blacklist": false,
                  "risk_factors": [{"name": "exchange", "label": "Exchange", "group": "low", "score": "0.62"}],
                  "checked_at": "2026-08-14 09:30:00"})),
    )
    .await;

    let check = client(&server).check_aml_status("aml-1").await.unwrap();
    assert_eq!(only_request(&server).await, json!({"id": "aml-1"}));
    assert_eq!(check.status, AmlStatus::Completed);
    assert_eq!(check.risk_score.unwrap().to_string(), "35.3");
    assert_eq!(check.risk_level, Some(AmlRiskLevel::Medium));
    assert_eq!(check.risk_factors[0].score.to_string(), "0.62");
    assert_eq!(check.checked_at.unwrap().unix_timestamp(), Some(1_786_699_800));
}

#[tokio::test]
async fn get_aml_history_defaults() {
    let server = MockServer::start().await;
    respond(
        &server,
        "/v1/aml-checks/history",
        ok(json!({"page": 1, "per_page": 10, "total": 0, "items": []})),
    )
    .await;

    let history = client(&server).get_aml_history(&AmlHistoryRequest::new()).await.unwrap();
    assert_eq!(only_request(&server).await, json!({"page": 1, "per_page": 10}));
    assert_eq!((history.page, history.per_page, history.total), (1, 10, 0));
    assert!(history.items.is_empty());
}

#[tokio::test]
async fn get_aml_history_filtered() {
    let server = MockServer::start().await;
    respond(
        &server,
        "/v1/aml-checks/history",
        ok(json!({"page": 2, "per_page": 50, "total": 51, "items": [{"id": "aml-51", "status": "completed"}]})),
    )
    .await;

    let request = AmlHistoryRequest::new().page(2).per_page(50).status(AmlStatus::Completed);
    let history = client(&server).get_aml_history(&request).await.unwrap();
    assert_eq!(only_request(&server).await, json!({"page": 2, "per_page": 50, "status": "completed"}));
    assert_eq!(history.items[0].id, "aml-51");
}

// A raw body: json! would sort the plan keys.
fn raw_ok(result: &str) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_raw(format!(r#"{{"code":0,"result":{result}}}"#), "application/json")
}

fn subscription(status: &str) -> serde_json::Value {
    json!({
        "id": "01m4e1z3q0r7x225zc6p63m5ey",
        "subscription_id": "unlimited_energy",
        "created_at": "2026-10-08T15:26:32+00:00",
        "expire_at": "2026-11-07T15:26:32+00:00",
        "address": "TAddress",
        "status": status,
        "external_id": "sub-1",
        "params": {"address": "TAddress", "duration": 30, "transactions_limit": 0, "activate_address": false}
    })
}

#[tokio::test]
async fn get_subscriptions_keeps_the_api_order() {
    let server = MockServer::start().await;
    respond(
        &server,
        "/v1/subscriptions",
        raw_ok(
            r#"{"unlimited_energy":{"id":8,"name":"Unlimited Energy","activation_fee":0,"initial_price":8,"price":2.8,"transactions_limit":0,"duration_days":0},
                "energy_pack_100":{"id":2,"name":"Energy Pack 100","activation_fee":"2.0","initial_price":10,"price":5,"transactions_limit":10,"duration_days":5}}"#,
        ),
    )
    .await;

    let plans = client(&server).get_subscriptions().await.unwrap();
    assert_eq!(only_request(&server).await, json!({}));
    assert_eq!(plans.len(), 2);
    let first = &plans[0];
    assert_eq!(
        (first.subscription_id.as_str(), first.id, first.name.as_str()),
        ("unlimited_energy", 8, "Unlimited Energy")
    );
    assert_eq!(
        (first.activation_fee.to_string(), first.initial_price.to_string(), first.price.to_string()),
        ("0".to_owned(), "8".to_owned(), "2.8".to_owned())
    );
    assert_eq!((first.transactions_limit, first.duration_days), (0, 0));
    let second = &plans[1];
    assert_eq!((second.subscription_id.as_str(), second.id), ("energy_pack_100", 2));
    assert_eq!(second.activation_fee.to_string(), "2.0");
    assert_eq!((second.transactions_limit, second.duration_days), (10, 5));
}

#[tokio::test]
async fn get_subscriptions_empty() {
    for result in ["{}", "[]"] {
        let server = MockServer::start().await;
        respond(&server, "/v1/subscriptions", raw_ok(result)).await;
        assert!(client(&server).get_subscriptions().await.unwrap().is_empty(), "{result}");
    }
}

#[tokio::test]
async fn get_subscriptions_rejects_unexpected_result() {
    for result in [r#""unlimited_energy""#, "5", r#"{"unlimited_energy":"cheap"}"#] {
        let server = MockServer::start().await;
        respond(&server, "/v1/subscriptions", raw_ok(result)).await;
        match client(&server).get_subscriptions().await {
            Err(TronzapError::Serialization(e)) => assert_eq!(e.message(), "unexpected result in response"),
            other => panic!("expected a serialization error for {result}, got {other:?}"),
        }
    }
}

#[tokio::test]
async fn get_subscriptions_reports_api_errors() {
    let server = MockServer::start().await;
    respond(
        &server,
        "/v1/subscriptions",
        ResponseTemplate::new(200).set_body_json(json!({"code": 6, "error": "Insufficient funds"})),
    )
    .await;
    assert_eq!(common::api_error(client(&server).get_subscriptions().await).code(), Some(6));
}

#[tokio::test]
async fn start_subscription() {
    let server = MockServer::start().await;
    respond(&server, "/v1/subscription/start", ok(subscription("active"))).await;

    let request = StartSubscriptionRequest::new("unlimited_energy", "TAddress")
        .duration_days(30)
        .external_id("sub-1")
        .activate_address(true);
    let sub = client(&server).start_subscription(&request).await.unwrap();
    assert_eq!(
        only_request(&server).await,
        json!({
            "subscription_id": "unlimited_energy",
            "external_id": "sub-1",
            "params": {"address": "TAddress", "duration": 30, "transactions_limit": 0, "activate_address": true}
        })
    );
    assert_eq!(sub.id, "01m4e1z3q0r7x225zc6p63m5ey");
    assert_eq!(sub.subscription_id, "unlimited_energy");
    assert_eq!(sub.external_id.as_deref(), Some("sub-1"));
    assert_eq!(sub.address.as_deref(), Some("TAddress"));
    assert_eq!(sub.status, SubscriptionStatus::Active);
    let params = sub.params.unwrap();
    assert_eq!(
        (params.address.as_str(), params.duration_days, params.transactions_limit),
        ("TAddress", 30, 0)
    );
    assert!(!params.activate_address);
    assert_eq!(sub.expire_at.unwrap().unix_timestamp(), Some(1_794_065_192));
    assert_eq!(sub.created_at.unwrap().unix_timestamp(), Some(1_791_473_192));
    assert_eq!((sub.stopped_at, sub.started_at, sub.renewed_at), (None, None, None));
}

#[tokio::test]
async fn start_subscription_sends_zero_limits() {
    let server = MockServer::start().await;
    respond(&server, "/v1/subscription/start", ok(subscription("active"))).await;

    let request = StartSubscriptionRequest::new("unlimited_energy", "TAddress");
    client(&server).start_subscription(&request).await.unwrap();
    assert_eq!(
        only_request(&server).await,
        json!({
            "subscription_id": "unlimited_energy",
            "params": {"address": "TAddress", "duration": 0, "transactions_limit": 0}
        })
    );
}

#[tokio::test]
async fn check_subscription_by_id() {
    let server = MockServer::start().await;
    respond(&server, "/v1/subscription/check", ok(subscription("active"))).await;

    let sub = client(&server).check_subscription(&SubscriptionRequest::by_id("sub-1")).await.unwrap();
    assert_eq!(only_request(&server).await, json!({"id": "sub-1"}));
    assert_eq!(sub.status, SubscriptionStatus::Active);
}

#[tokio::test]
async fn check_subscription_by_external_id() {
    let server = MockServer::start().await;
    respond(&server, "/v1/subscription/check", ok(subscription("paused"))).await;

    let request = SubscriptionRequest::by_external_id("pedido-año-订单-😀");
    let sub = client(&server).check_subscription(&request).await.unwrap();
    assert_eq!(only_request(&server).await, json!({"external_id": "pedido-año-订单-😀"}));
    assert_eq!(sub.status, SubscriptionStatus::Unknown("paused".into()));
}

#[tokio::test]
async fn stop_subscription_without_address_and_expiry() {
    let server = MockServer::start().await;
    respond(
        &server,
        "/v1/subscription/stop",
        ok(json!({
            "id": "01m4e1z3q0r7x225zc6p63m5ey", "subscription_id": "unlimited_energy",
            "created_at": "2026-10-08T15:26:32+00:00", "stopped_at": "2026-10-08T15:28:44+00:00",
            "status": "stopped", "external_id": null,
            "params": {"address": "TAddress", "duration": 30, "transactions_limit": 0, "activate_address": false,
                       "future_field": 1}
        })),
    )
    .await;

    let request = SubscriptionRequest::by_id("01m4e1z3q0r7x225zc6p63m5ey").external_id("sub-1");
    let sub = client(&server).stop_subscription(&request).await.unwrap();
    assert_eq!(
        only_request(&server).await,
        json!({"id": "01m4e1z3q0r7x225zc6p63m5ey", "external_id": "sub-1"})
    );
    assert_eq!(sub.status, SubscriptionStatus::Stopped);
    assert_eq!((sub.external_id, sub.address, sub.expire_at), (None, None, None));
    assert_eq!(sub.stopped_at.unwrap().unix_timestamp(), Some(1_791_473_324));
    assert_eq!(sub.params.unwrap().address, "TAddress");
}

#[tokio::test]
async fn get_subscription_history_defaults() {
    let server = MockServer::start().await;
    respond(
        &server,
        "/v1/subscriptions/history",
        ok(json!({"page": 1, "per_page": 10, "total": 1, "items": [{
            "id": "01m4e1z3q0r7x225zc6p63m5ey", "status": "active", "subscription_id": "unlimited_energy",
            "address": "TAddress", "transactions_limit": 0, "transactions_used": 4, "energy_used": 262000,
            "total_price": 13.6, "started_at": "2026-10-08T15:26:33+00:00",
            "renewed_at": "2026-10-08T15:27:35+00:00", "stopped_at": null,
            "expire_at": "2026-11-07T15:26:32+00:00", "created_at": "2026-10-08T15:26:32+00:00"
        }]})),
    )
    .await;

    let history = client(&server).get_subscription_history(&SubscriptionHistoryRequest::new()).await.unwrap();
    assert_eq!(only_request(&server).await, json!({"page": 1, "per_page": 10}));
    assert_eq!((history.page, history.per_page, history.total), (1, 10, 1));
    let item = &history.items[0];
    assert_eq!((item.transactions_limit, item.transactions_used, item.energy_used), (0, 4, 262_000));
    assert_eq!(item.total_price.to_string(), "13.6");
    assert_eq!(item.params, None);
    assert_eq!(item.renewed_at.as_ref().unwrap().unix_timestamp(), Some(1_791_473_255));
    assert!(item.started_at.is_some());
    assert_eq!(item.stopped_at, None);
}

#[tokio::test]
async fn get_subscription_history_filtered_with_string_price() {
    let server = MockServer::start().await;
    respond(
        &server,
        "/v1/subscriptions/history",
        ok(json!({"page": 2, "per_page": 50, "total": 51, "items": [{"id": "sub-51", "total_price": "8.00"}]})),
    )
    .await;

    let request = SubscriptionHistoryRequest::new().page(2).per_page(50).status(SubscriptionStatus::Active);
    let history = client(&server).get_subscription_history(&request).await.unwrap();
    assert_eq!(only_request(&server).await, json!({"page": 2, "per_page": 50, "status": "active"}));
    assert_eq!(history.items[0].total_price.to_string(), "8.00");
}

#[tokio::test]
async fn raw_request_to_any_endpoint() {
    let server = MockServer::start().await;
    respond(&server, "/v1/custom", ok(json!({"ok": true}))).await;

    let result: serde_json::Value = client(&server).request("v1/custom", &json!({"x": 1})).await.unwrap();
    assert_eq!(only_request(&server).await, json!({"x": 1}));
    assert_eq!(result, json!({"ok": true}));
}

#[tokio::test]
async fn base_url_with_path_prefix() {
    let server = MockServer::start().await;
    respond(&server, "/proxy/v1/balance", ok(json!({"balance": 1}))).await;

    let client = tronzap_sdk::TronzapClient::builder()
        .api_token(common::TOKEN)
        .api_secret(common::SECRET)
        .base_url(format!("{}/proxy/", server.uri()))
        .build()
        .unwrap();
    assert_eq!(client.get_balance().await.unwrap().balance.to_string(), "1");
}

#[tokio::test]
async fn concurrent_calls_share_one_client() {
    let server = MockServer::start().await;
    respond(&server, "/v1/balance", ok(json!({"balance": "1"}))).await;
    let client = client(&server);

    let tasks: Vec<_> = (0..32)
        .map(|_| {
            let client = client.clone();
            tokio::spawn(async move { client.get_balance().await })
        })
        .collect();
    for task in tasks {
        assert_eq!(task.await.unwrap().unwrap().balance.to_string(), "1");
    }

    let requests = server.received_requests().await.unwrap();
    assert_eq!(requests.len(), 32);
}

#[tokio::test]
async fn signs_a_non_ascii_request_body() {
    let server = MockServer::start().await;
    respond(
        &server,
        "/v1/transaction/new",
        ok(transaction("energy", json!({"address": ADDRESS, "amounts": {"energy": 65000}, "duration": 1}))),
    )
    .await;

    let request = EnergyTransactionRequest::new(ADDRESS, 65000).external_id("pedido-año-订单-😀");
    client(&server).create_energy_transaction(&request).await.unwrap();
    assert_eq!(only_request(&server).await["external_id"], "pedido-año-订单-😀");
}
