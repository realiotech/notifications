use dotenv;
use ethers::etherscan::Client;
use ethers::prelude::*;
use eyre::Result;
use reqwest::header::{ACCEPT, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use slack_hook::{PayloadBuilder, Slack};
use std::env;
use tokio::time::{interval, Duration};
use tokio::{spawn, task};
use tracing::*;

// use std::{convert::TryFrom, path::Path, sync::Arc, time::Duration};

abigen!(
    RioToken,
    "./config/rio.json",
    event_derives(serde::Deserialize, serde::Serialize)
);

struct Channel {
    name: String,
    address: H160,
}

extern crate serde_derive;

#[derive(Debug, Serialize, Deserialize)]
pub struct Welcome {
    #[serde(rename = "_links")]
    links: Links,
    horizon_version: String,
    core_version: String,
    ingest_latest_ledger: i64,
    history_latest_ledger: i64,
    history_latest_ledger_closed_at: String,
    history_elder_ledger: i64,
    core_latest_ledger: i64,
    network_passphrase: String,
    current_protocol_version: i64,
    supported_protocol_version: i64,
    core_supported_protocol_version: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Links {
    account: Account,
    accounts: Account,
    account_transactions: Account,
    claimable_balances: Account,
    assets: Account,
    effects: Account,
    fee_stats: FeeStats,
    ledger: Account,
    ledgers: Account,
    liquidity_pools: Account,
    offer: Account,
    offers: Account,
    operation: Account,
    operations: Account,
    order_book: Account,
    payments: Account,
    #[serde(rename = "self")]
    links_self: FeeStats,
    strict_receive_paths: Account,
    strict_send_paths: Account,
    trade_aggregations: Account,
    trades: Account,
    transaction: Account,
    transactions: Account,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    href: String,
    templated: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeeStats {
    href: String,
}

async fn get_stellar_sync_state() -> bool {
    let client = reqwest::Client::new();
    let response_realio = client
        .clone()
        .get(&env::var("REALIO_STELLAR_NODE").unwrap())
        .header(CONTENT_TYPE, "application/json")
        .header(ACCEPT, "application/json")
        .header("pragma", "public")
        .send()
        .await
        .unwrap()
        .json::<Welcome>()
        .await
        .unwrap();

    let realio_latest_block: i64 = response_realio.ingest_latest_ledger;

    let response_horizon = client
        .clone()
        .get(&env::var("HORIZON_STELLAR_NODE").unwrap())
        .header(CONTENT_TYPE, "application/json")
        .header(ACCEPT, "application/json")
        .header("pragma", "public")
        .send()
        .await
        .unwrap()
        .json::<Welcome>()
        .await
        .unwrap();

    let horizon_latest_block: i64 = response_horizon.ingest_latest_ledger;
    let sync_status: bool;
    if realio_latest_block >= horizon_latest_block - 2 {
        sync_status = true
    } else {
        sync_status = false
    };
    sync_status
}

async fn get_eth_balance() -> Result<()> {
    let client = Client::new(
        Chain::Mainnet,
        env::var("ETHERSCAN_API_KEY").expect("Etherscan API Key not found"),
    );
    let reserve_1 = "0xdf6764f41eb550f060aea6c852b822a49b53c6e2"
        .parse::<Address>()
        .expect("Unable to parse address");
    let reserve_2 = "0xb47936bbc484e56dda8b57440ca3c5459f495cf3"
        .parse::<Address>()
        .expect("Unable to parse address");
    let reserve_3 = "0x0133f9f460282d10c8db2d3376764412af57808c"
        .parse::<Address>()
        .expect("Unable to parse address");
    let reserve_4 = "0xa49a964d5345a9c8f7f6516c5aaee7a3dd1b7870"
        .parse::<Address>()
        .expect("Unable to parse address");
    let reserve_5 = "0x94c3857520e9151b34814fbf8b477368f4a97ea7"
        .parse::<Address>()
        .expect("Unable to parse address");
    let reserve_6 = "0x914f1f73f42c3aca3328d41210e32731a7f969c8"
        .parse::<Address>()
        .expect("Unable to parse address");
    let reserve_7 = "0x6005121a46bb3028872cf471faab92a08b2d0f5a"
        .parse::<Address>()
        .expect("Unable to parse address");
    let reserve_8 = "0x8e385bc51f7a5385604d8617c9ba2a40f9e5a387"
        .parse::<Address>()
        .expect("Unable to parse address");
    let reserve_9 = "0xab112ddda6d0196915618d605d909306e7c7ebd7"
        .parse::<Address>()
        .expect("Unable to parse address");
    let reserve_10 = "0x5e1c7f0ef930d79598f9fadbaca3c1bea400e6f7"
        .parse::<Address>()
        .expect("Unable to parse address");
    let balances = client
        .expect("foo")
        .get_ether_balance_multi(
            &vec![
                &reserve_1,
                &reserve_2,
                &reserve_3,
                &reserve_4,
                &reserve_5,
                &reserve_6,
                &reserve_7,
                &reserve_8,
                &reserve_9,
                &reserve_10,
            ],
            None,
        )
        .await
        .unwrap();

    let slack = Slack::new(
        env::var("SLACK_WEBHOOK_URL")
            .expect("Etherscan API Key not found")
            .as_str(),
    )
    .unwrap();

    let mut low_balances = balances
        .iter()
        .filter(|n| {
            (n.balance.parse::<f64>().unwrap()) / 1000000000000000000.0
                < "0.1".parse::<f64>().unwrap()
        })
        .collect::<Vec<_>>();
    for element in low_balances.iter_mut() {
        println!("Address {:?}", element.account);
        let p = PayloadBuilder::new()
            .text(format!(
                "ETH Balance in Account https://etherscan.io/address/{:?}  is low, current amount is {} ETH .. please feed me @Derek\n
                Sync Status {:?}",
                element.account,
                element.balance.parse::<f64>().unwrap() / 1000000000000000000.0, get_stellar_sync_state().await
            ))
            .channel("#balances_bot")
            .username("Balances Bot")
            .icon_emoji(":eyes:")
            .build()
            .unwrap();

        let res = slack.send(&p);
        match res {
            Ok(()) => println!("ok"),
            Err(x) => println!("ERR: {:?}", x),
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();
    let forever = task::spawn(async {
        let mut interval = interval(Duration::from_secs(600));

        loop {
            interval.tick().await;
            tracing::info!("Calling balance");
            get_eth_balance().await.unwrap();
        }
    });

    forever.await.unwrap();
}
