//! Walks through the TronZap API operations. By default it only reads and spends nothing.
//!
//! ```sh
//! export TRONZAP_API_TOKEN=your_api_token
//! export TRONZAP_API_SECRET=your_api_secret
//! export TRONZAP_BASE_URL=api.tronzap.com      # optional, e.g. a dev host
//! export TRONZAP_ADDRESS=TRON_ADDRESS          # optional
//! export TRONZAP_FROM_ADDRESS=TRON_ADDRESS     # optional, with TO_ADDRESS
//! export TRONZAP_TO_ADDRESS=TRON_ADDRESS       # optional, with FROM_ADDRESS
//! export TRONZAP_TRANSACTION_ID=id             # optional
//! export TRONZAP_AML_CHECK_ID=id               # optional
//! export TRONZAP_SUBSCRIPTION_ID=id            # optional
//! cargo run --example basic_usage
//! ```
//!
//! Setting `TRONZAP_ALLOW_PURCHASES=1` also exercises the endpoints that create
//! transactions and AML checks. Those DEBIT THE ACCOUNT BALANCE. It is meant for
//! verifying an integration against a development environment, and it also needs
//! `TRONZAP_ADDRESS`.
//!
//! Setting `TRONZAP_SUBSCRIPTION_PLAN` (a plan key such as `unlimited_energy`) as
//! well starts a one-day subscription to that plan for `TRONZAP_ADDRESS` and stops
//! it straight away. Starting one CHARGES THE PLAN'S INITIAL PRICE.

use std::process::ExitCode;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tronzap_sdk::models::Timestamp;
use tronzap_sdk::requests::{
    AddressActivationRequest, AmlCheckRequest, AmlHistoryRequest, BandwidthTransactionRequest,
    CalculateRequest, CheckTransactionRequest, EnergyTransactionRequest, EstimateEnergyRequest,
    ResourceBundleTransactionRequest, StartSubscriptionRequest, SubscriptionHistoryRequest,
    SubscriptionRequest,
};
use tronzap_sdk::responses::{Subscription, Transaction};
use tronzap_sdk::{ErrorCode, TronzapClient, TronzapError};

const ENERGY: u64 = 65000;
const BANDWIDTH: u64 = 345;

#[tokio::main]
async fn main() -> ExitCode {
    let (Some(token), Some(secret)) = (env("TRONZAP_API_TOKEN"), env("TRONZAP_API_SECRET")) else {
        eprintln!("set TRONZAP_API_TOKEN and TRONZAP_API_SECRET");
        return ExitCode::from(2);
    };

    let mut builder = TronzapClient::builder()
        .api_token(token)
        .api_secret(secret)
        .timeout(Duration::from_secs(20))
        .user_agent("tronzap-example/1.0");
    if let Some(base_url) = env("TRONZAP_BASE_URL") {
        builder = builder.base_url(base_url);
    }
    let client = match builder.build() {
        Ok(client) => client,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(2);
        }
    };
    println!("Calling {}", client.base_url());

    let mut run = Run::default();

    run.step("get_balance", client.get_balance().await, |b| {
        println!("  balance {}, deposit address {}", b.balance, b.address);
    });
    run.step("get_services", client.get_services().await, |s| {
        for rate in &s.energy {
            println!(
                "  energy {}h {}..{} at {} per 1000 units (65k = {})",
                rate.duration, rate.min_amount, rate.max_amount, rate.price, rate.price_65k
            );
        }
        for rate in &s.bandwidth {
            println!(
                "  bandwidth {}h {}..{} at {} per 1000 units",
                rate.duration, rate.min_amount, rate.max_amount, rate.price
            );
        }
        if let Some(activation) = &s.activate_address {
            println!("  activation {}", activation.price);
        }
    });
    run.step("get_direct_recharge_info", client.get_direct_recharge_info().await, |i| {
        println!("  pay to {}, {} rate(s)", i.address, i.rates.len());
    });
    run.step("get_aml_services", client.get_aml_services().await, |services| {
        for service in services {
            println!("  {} {} at {}", service.id, service.check_type, service.price);
        }
    });
    run.step("get_aml_history", client.get_aml_history(&AmlHistoryRequest::new()).await, |h| {
        println!("  page {}, {} of {} check(s)", h.page, h.items.len(), h.total);
    });

    run.step("get_subscriptions", client.get_subscriptions().await, |plans| {
        for plan in plans {
            println!(
                "  {} ({}): activation {}, initial {}, {} per transaction, limit {} transactions, {} days",
                plan.subscription_id,
                plan.name,
                plan.activation_fee,
                plan.initial_price,
                plan.price,
                plan.transactions_limit,
                plan.duration_days
            );
        }
    });
    let request = SubscriptionHistoryRequest::new().per_page(3);
    run.step("get_subscription_history", client.get_subscription_history(&request).await, |h| {
        println!("  page {}, {} of {} subscription(s)", h.page, h.items.len(), h.total);
        for sub in &h.items {
            println!(
                "  {} {} {}, {} transaction(s), {} energy, charged {}, expires {}",
                sub.id,
                sub.subscription_id,
                sub.status,
                sub.transactions_used,
                sub.energy_used,
                sub.total_price,
                describe(sub.expire_at.as_ref())
            );
        }
    });

    let address = env("TRONZAP_ADDRESS");
    match &address {
        Some(address) => {
            run.step("get_address_info", client.get_address_info(address).await, |i| {
                let balances: Vec<String> = i.balances.iter().map(|(k, v)| format!("{k} {v}")).collect();
                println!(
                    "  energy {}, bandwidth {}, balances {}",
                    i.resources.energy,
                    i.resources.bandwidth,
                    balances.join(", ")
                );
            });
            let request = CalculateRequest::new(address.as_str(), ENERGY);
            run.step("calculate", client.calculate(&request).await, |c| {
                println!("  {} energy for {}h costs {}", c.amount, c.duration, c.total);
            });
        }
        None => {
            Run::skip("get_address_info");
            Run::skip("calculate");
        }
    }

    match (env("TRONZAP_FROM_ADDRESS"), env("TRONZAP_TO_ADDRESS")) {
        (Some(from), Some(to)) => {
            let request = EstimateEnergyRequest::new(from, to);
            run.step("estimate_energy", client.estimate_energy(&request).await, |e| {
                println!("  {} energy, total {}", e.amount, e.total);
            });
        }
        _ => Run::skip("estimate_energy"),
    }

    match env("TRONZAP_TRANSACTION_ID") {
        Some(id) => {
            let request = CheckTransactionRequest::by_id(id);
            run.step("check_transaction", client.check_transaction(&request).await, print_transaction);
        }
        None => Run::skip("check_transaction"),
    }

    match env("TRONZAP_AML_CHECK_ID") {
        Some(id) => run.step("check_aml_status", client.check_aml_status(&id).await, |c| {
            let risk = c.risk_score.map_or_else(|| "not scored yet".to_owned(), |s| s.to_string());
            println!("  {}, risk {risk}", c.status);
        }),
        None => Run::skip("check_aml_status"),
    }

    match env("TRONZAP_SUBSCRIPTION_ID") {
        Some(id) => {
            let request = SubscriptionRequest::by_id(id);
            run.step("check_subscription", client.check_subscription(&request).await, print_subscription);
        }
        None => Run::skip("check_subscription"),
    }

    if env("TRONZAP_ALLOW_PURCHASES").as_deref() != Some("1") {
        println!(
            "\nSkipping purchases: set TRONZAP_ALLOW_PURCHASES=1 to create transactions (debits the balance)"
        );
    } else if let Some(address) = &address {
        let run_id = format!(
            "rust-example-{}",
            SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_millis())
        );

        let request =
            AddressActivationRequest::new(address.as_str()).external_id(format!("{run_id}-activate"));
        match client.create_address_activation_transaction(&request).await {
            Err(TronzapError::Api(e)) if e.error_code() == Some(ErrorCode::AddressAlreadyActivated) => {
                println!("\ncreate_address_activation_transaction\n  already activated");
            }
            result => run.step("create_address_activation_transaction", result, print_transaction),
        }

        let energy_id = format!("{run_id}-energy");
        let request = EnergyTransactionRequest::new(address.as_str(), ENERGY).external_id(energy_id.as_str());
        run.step(
            "create_energy_transaction",
            client.create_energy_transaction(&request).await,
            print_transaction,
        );
        let request = CheckTransactionRequest::by_external_id(energy_id);
        run.step("check_transaction", client.check_transaction(&request).await, print_transaction);

        let request = BandwidthTransactionRequest::new(address.as_str(), BANDWIDTH)
            .external_id(format!("{run_id}-bandwidth"));
        run.step(
            "create_bandwidth_transaction",
            client.create_bandwidth_transaction(&request).await,
            print_transaction,
        );

        let request = ResourceBundleTransactionRequest::new(address.as_str(), ENERGY, BANDWIDTH)
            .external_id(format!("{run_id}-bundle"));
        run.step(
            "create_resource_bundle_transaction",
            client.create_resource_bundle_transaction(&request).await,
            print_transaction,
        );

        let request = AmlCheckRequest::for_address("TRX", address.as_str());
        run.step("create_aml_check", client.create_aml_check(&request).await, |c| {
            println!("  AML check {} is {}", c.id, c.status);
        });

        match env("TRONZAP_SUBSCRIPTION_PLAN") {
            Some(plan) => {
                let request = StartSubscriptionRequest::new(plan, address.as_str())
                    .duration_days(1)
                    .external_id(format!("{run_id}-subscription"));
                let mut started = None;
                run.step("start_subscription", client.start_subscription(&request).await, |sub| {
                    started = Some(sub.id.clone());
                    print_subscription(sub);
                });
                if let Some(id) = started {
                    let request = SubscriptionRequest::by_id(id);
                    run.step(
                        "check_subscription",
                        client.check_subscription(&request).await,
                        print_subscription,
                    );
                    run.step(
                        "stop_subscription",
                        client.stop_subscription(&request).await,
                        print_subscription,
                    );
                }
            }
            None => Run::skip("start_subscription"),
        }
    } else {
        println!("\nSkipping purchases: TRONZAP_ADDRESS is not set");
    }

    if run.failed.is_empty() {
        println!("\nAll calls succeeded");
        ExitCode::SUCCESS
    } else {
        eprintln!("\nFailed: {}", run.failed.join(", "));
        ExitCode::FAILURE
    }
}

#[derive(Default)]
struct Run {
    failed: Vec<&'static str>,
}

impl Run {
    fn step<T>(&mut self, name: &'static str, result: Result<T, TronzapError>, print: impl FnOnce(T)) {
        println!("\n{name}");
        match result {
            Ok(value) => print(value),
            Err(e) => {
                println!("  FAILED: {e}");
                self.failed.push(name);
            }
        }
    }

    fn skip(name: &str) {
        println!("\n{name}\n  skipped: its environment variable is not set");
    }
}

fn print_transaction(tx: Transaction) {
    println!(
        "  {} {} {}, charged {}, created {}",
        tx.id,
        tx.service,
        tx.status,
        tx.amount,
        describe(tx.created_at.as_ref())
    );
}

fn print_subscription(sub: Subscription) {
    println!(
        "  {} {} {}, address {}, created {}, expires {}, stopped {}",
        sub.id,
        sub.subscription_id,
        sub.status,
        sub.address.as_deref().unwrap_or("not reported"),
        describe(sub.created_at.as_ref()),
        describe(sub.expire_at.as_ref()),
        describe(sub.stopped_at.as_ref())
    );
}

fn describe(timestamp: Option<&Timestamp>) -> String {
    match timestamp {
        None => "unknown".to_owned(),
        Some(t) => match t.unix_timestamp() {
            Some(seconds) => format!("{seconds} (unix seconds)"),
            None => format!("UNPARSED({t})"),
        },
    }
}

fn env(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|v| !v.trim().is_empty())
}
