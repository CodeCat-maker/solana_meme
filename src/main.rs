// 导入 `colored` 包，用于在终端输出中添加颜色，使输出更具可读性。
// Import the `colored` crate to add colors to terminal output, making output more readable.
use colored::*;

// 导入 `futures_util` 的 `StreamExt` 特征，它为流提供了异步迭代等扩展方法。
// Import `StreamExt` trait from `futures_util`, which provides extra methods like async iteration for streams.
use futures_util::StreamExt;

// 导入 Solana 客户端库。
// Import Solana client library.
use solana_client;

// 从 `solana_client` 导入 `PubsubClient`，它是非阻塞 WebSocket 客户端，用于订阅实时事件。
// Import `PubsubClient` from `solana_client`, a non-blocking WebSocket client for subscribing to real-time events.
use solana_client::nonblocking::pubsub_client::PubsubClient;

// 从 `solana_client` 导入 `rpc_client`，用于通过 RPC 与节点交互。
// Import `rpc_client` from `solana_client`, for interacting with nodes via RPC.
use solana_client::rpc_client;

// 导入 RPC 配置相关模块。
// Import RPC configuration related modules.
use solana_client::rpc_config::RpcTransactionConfig;
use solana_client::rpc_config::RpcTransactionLogsConfig;
use solana_client::rpc_config::RpcTransactionLogsFilter;

// 导入 `CommitmentConfig`，用于设置查询确认级别。
// Import `CommitmentConfig`, used to set the query commitment level.
use solana_sdk::commitment_config::CommitmentConfig;

// 导入 `Pubkey`，表示 Solana 公钥。
// Import `Pubkey`, representing a Solana public key.
use solana_sdk::pubkey::Pubkey;

// 导入 `signature` 模块。
// Import the `signature` module.
use solana_sdk::signature;

// 导入 `Signature`，表示交易签名。
// Import `Signature`, representing a transaction signature.
use solana_sdk::signature::Signature;

// 导入 `transaction` 模块。
// Import the `transaction` module.
use solana_sdk::transaction;

// 导入 `UiTransactionEncoding`，指定交易返回编码格式。
// Import `UiTransactionEncoding`, specifying the transaction response encoding format.
use solana_transaction_status::UiTransactionEncoding;

// 导入 `std::mem`，虽然此处未用到，但可能用于内存操作。
// Import `std::mem`, although not used here, possibly for memory operations.
use std::mem;

// 导入 `FromStr` 特征，用于将字符串解析为特定类型。
// Import `FromStr` trait, used to parse strings into specific types.
use std::str::FromStr;

// 导入 `Url`，用于处理 URL。
// Import `Url`, used for handling URLs.
use url::Url;

// 定义环境配置结构体，用于存储 WebSocket URL。
// Define environment configuration struct for storing WebSocket URL.
#[derive(serde::Deserialize)]
struct Env {
    // WebSocket URL。
    // WebSocket URL.
    ws_url: url::Url,
}

// 主入口函数，异步执行。
// Main entry function, runs asynchronously.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化环境变量，硬编码 WebSocket URL，实际应从配置加载。
    // Initialize environment variables, WebSocket URL hardcoded here, should load from config in real usage.
    let env = Env {
        ws_url: Url::parse("wss://mainnet.helius-rpc.com/?api-key=YOUR_API_KEY_HERE")?,
    };

    // 创建 PubsubClient，用于订阅链上事件。
    // Create PubsubClient for subscribing to on-chain events.
    let ps_client = PubsubClient::new(&env.ws_url.to_string()).await?;

    // 创建 RpcClient，用于查询链上数据。
    // Create RpcClient for querying on-chain data.
    let rpc_client = rpc_client::RpcClient::new("https://mainnet.helius-rpc.com/?api-key=YOUR_API_KEY_HERE".to_string());

    // 创建要监听的程序公钥。
    // Create the program public key to listen to.
    let ray_fee_pubkey = Pubkey::from_str("7YttLkHDoNj9wyDur5pM1ejNaAvT9X4eqaYcHQqtj2G5")?;

    // 创建日志订阅过滤器，只接收包含指定公钥的交易。
    // Create log subscription filter, only receive transactions mentioning the specified public key.
    let filters = RpcTransactionLogsFilter::Mentions(vec![ray_fee_pubkey.to_string()]);

    // 创建订阅配置，设置确认级别。
    // Create subscription config, set commitment level.
    let config = RpcTransactionLogsConfig {
        commitment: Some(CommitmentConfig::confirmed()),
    };

    // 开始订阅日志流。
    // Start subscribing to log stream.
    let (mut logs_stream, unsubscriber) = ps_client.logs_subscribe(filters, config).await?;

    // 输出监控开始信息。
    // Print monitoring started message.
    println!("开始监控...");
    println!("Monitoring started...");

    // 异步循环处理日志流。
    // Async loop to process log stream.
    while let Some(response) = logs_stream.next().await {
        // 获取响应值。
        // Get response value.
        let value = response.value;

        // 获取交易签名。
        // Get transaction signature.
        let signature = value.signature;

        // 签名转换为字符串。
        // Convert signature to string.
        let transaction_signature_str = signature.as_str();

        // 将字符串解析为 Signature 类型。
        // Parse string into Signature type.
        let transaction_signature = Signature::from_str(transaction_signature_str)
            .map_err(|e| format!("签名错误: {}", e))?;

        // 创建查询交易的配置。
        // Create config for querying transaction.
        let config = RpcTransactionConfig {
            encoding: Some(UiTransactionEncoding::Json),
            max_supported_transaction_version: Some(0),
            commitment: Some(CommitmentConfig::confirmed()),
        };

        // 获取交易详情。
        // Get transaction details.
        let transaction = rpc_client.get_transaction_with_config(&transaction_signature, config);

        // 处理交易元数据。
        // Process transaction metadata.
        transaction?.transaction.meta.map(|meta| {
            meta.post_token_balances.map(|token_balances: Vec<_>| {
                token_balances.iter().for_each(|token_balance| {
                    // 检查是否是指定 owner 且非 SOL。
                    // Check if owner matches specified and not SOL.
                    if matches!(&token_balance.owner, solana_transaction_status::option_serializer::OptionSerializer::Some(s) if s == "5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1")
                        && token_balance.mint != "So11111111111111111111111111111111111111112"
                    {
                        // 打印新币信息。
                        // Print new token information.
                        println!("{}", "========== 发现新币 ==========".bold().blue());
                        println!("{}", "========== New token found ==========".bold().blue());
                        println!("代币地址: {}", token_balance.mint.green());
                        println!("Token mint address: {}", token_balance.mint.green());
                        println!("{}", "=====================================".bold().blue());
                    }
                });
            });
        });
    }

    // 调用取消订阅函数。
    // Call the unsubscribe function.
    unsubscriber().await;

    // 返回 Ok 表示成功。
    // Return Ok indicating success.
    Ok(())
}
