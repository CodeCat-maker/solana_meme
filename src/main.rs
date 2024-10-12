use colored::*;
use futures_util::StreamExt;
use solana_client;
use solana_client::nonblocking::pubsub_client::PubsubClient;
use solana_client::rpc_client;
use solana_client::rpc_config::RpcTransactionConfig;
use solana_client::rpc_config::RpcTransactionLogsConfig;
use solana_client::rpc_config::RpcTransactionLogsFilter;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature;
use solana_sdk::signature::Signature;
use solana_sdk::transaction;
use solana_transaction_status::UiTransactionEncoding;
use std::mem;
use std::str::FromStr;
use url::Url; // 引入 FromStr 特征

#[derive(serde::Deserialize)]
struct Env {
    ws_url: url::Url,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let env = Env {
        ws_url: Url::parse("your wss link")?,
    };

    let ps_client = PubsubClient::new(&env.ws_url.to_string()).await?;
    let rpc_client = rpc_client::RpcClient::new("your rpc link".to_string());

    let ray_fee_pubkey = Pubkey::from_str("7YttLkHDoNj9wyDur5pM1ejNaAvT9X4eqaYcHQqtj2G5")?;

    let filters = RpcTransactionLogsFilter::Mentions(vec![ray_fee_pubkey.to_string()]);
    let config = RpcTransactionLogsConfig {
        commitment: Some(CommitmentConfig::confirmed()), // 设置为 confirmed 确认级别
    };
    let (mut logs_stream, unsubscriber) = ps_client.logs_subscribe(filters, config).await?;
    println!("Start moitoring...");
    while let Some(response) = logs_stream.next().await {
        let value = response.value;
        let signature = value.signature;
        let transaction_signature_str = signature.as_str();

        let transaction_signature = Signature::from_str(transaction_signature_str)
            .map_err(|e| format!("Invalid signature: {}", e))?;

        let config = RpcTransactionConfig {
            encoding: Some(UiTransactionEncoding::Json),
            max_supported_transaction_version: Some(0), // 这里指定支持交易版本 0
            commitment: Some(CommitmentConfig::confirmed()),
        };

        let transaction = rpc_client.get_transaction_with_config(&transaction_signature, config);
        transaction?.transaction.meta.map(|meta| {
            meta.post_token_balances.map(|meta: Vec<_>| {
                meta.iter().for_each(|meta| {
                    let owner = meta.owner.as_ref().unwrap();
                    if owner == "5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1"
                        && meta.mint != "So11111111111111111111111111111111111111112"
                    {
                        println!("{}", "========== New Token Found ==========".bold().blue());
                        println!("Mint Address: {}", meta.mint.green());
                        println!("{}", "=====================================".bold().blue());
                    }
                });
            });
        });
    }

    unsubscriber().await;

    Ok(())
}
