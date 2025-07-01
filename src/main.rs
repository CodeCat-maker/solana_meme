// 导入 `colored` 包，用于在终端输出中添加颜色，使输出更具可读性。
use colored::*;
// 导入 `futures_util` 的 `StreamExt` 特征，它为流（Stream）提供了额外的方法，如 `.next()`，用于异步迭代。
use futures_util::StreamExt;
// 导入 `solana_client` 包，这是与 Solana 区块链交互的核心客户端库。
use solana_client;
// 从 `solana_client` 中导入 `PubsubClient`，这是一个非阻塞的 WebSocket 客户端，用于订阅 Solana 节点的实时事件。
use solana_client::nonblocking::pubsub_client::PubsubClient;
// 从 `solana_client` 中导入 `rpc_client`，用于通过 RPC (远程过程调用) 与 Solana 节点通信。
use solana_client::rpc_client;
// 导入 `RpcTransactionConfig`，用于在查询交易详情时指定配置，如编码格式。
use solana_client::rpc_config::RpcTransactionConfig;
// 导入 `RpcTransactionLogsConfig`，用于配置日志订阅，比如设置确认级别。
use solana_client::rpc_config::RpcTransactionLogsConfig;
// 导入 `RpcTransactionLogsFilter`，用于在订阅日志时设置过滤器，只接收我们感兴趣的交易日志。
use solana_client::rpc_config::RpcTransactionLogsFilter;
// 导入 `CommitmentConfig`，用于指定查询数据时所需的区块链确认级别（如 "processed", "confirmed", "finalized"）。
use solana_sdk::commitment_config::CommitmentConfig;
// 导入 `Pubkey`，代表 Solana 上的公钥（地址）。
use solana_sdk::pubkey::Pubkey;
// 导入 `signature` 模块。
use solana_sdk::signature;
// 导入 `Signature`，代表 Solana 上的交易签名。
use solana_sdk::signature::Signature;
// 导入 `transaction` 模块。
use solana_sdk::transaction;
// 导入 `UiTransactionEncoding`，定义了交易数据的编码格式，这里使用 JSON 格式。
use solana_transaction_status::UiTransactionEncoding;
// 导入 `std::mem`，虽然在此代码中未显式使用，但可能用于内存管理。
use std::mem;
// 导入 `FromStr` 特征，用于将字符串解析为特定类型，比如将字符串转换为 `Pubkey` 或 `Signature`。
use std::str::FromStr;
// 导入 `url::Url`，用于处理和验证 URL。
use url::Url; // 引入 FromStr 特征

// 定义一个名为 `Env` 的结构体，用于存放应用的环境变量。
// `#[derive(serde::Deserialize)]` 这个属性宏让 `Env` 结构体可以从序列化数据（如 JSON）中自动反序列化。
#[derive(serde::Deserialize)]
struct Env {
    // WebSocket 连接的 URL。
    ws_url: url::Url,
}

// 使用 `#[tokio::main]` 宏，将 `main` 函数设置为一个异步的运行时入口点。
// 这意味着我们可以在 `main` 函数中使用 `await` 关键字。
// 函数返回一个 `Result`，如果成功则返回 `()`，如果出错则返回一个包含错误信息的 `Box<dyn std::error::Error>`。
#[tokio::main]

//Box是指针分配到堆上，dyn是用来标识动态分发的 trait 对象，dyn 表示这个 trait 对象的具体类型在编译时不确定，而是在运行时决定
//这里的std::error::Error是 trait 对象，表示任何实现了 `Error` trait 的类型
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化 Env 结构体
    // 注意：这里的 WebSocket URL 是硬编码的，在实际应用中，建议从配置文件或环境变量中读取
    let env = Env {
        ws_url: Url::parse("wss://mainnet.helius-rpc.com/?api-key=9b999b3f-6772-48a1-8e29-b3418a74d702")?, // 请替换成你的 WebSocket URL
    };

    // 根据提供的 WebSocket URL 创建一个新的 `PubsubClient` 实例，用于订阅事件
    // `await` 用于等待异步操作完成
    let ps_client = PubsubClient::new(&env.ws_url.to_string()).await?;
    // 创建一个新的 `RpcClient` 实例，用于向 Solana 节点发送 RPC 请求
    // 注意：这里的 RPC URL 也是硬编码的
    let rpc_client = rpc_client::RpcClient::new("https://mainnet.helius-rpc.com/?api-key=9b999b3f-6772-48a1-8e29-b3418a74d702".to_string()); // 请替换成你的 RPC URL

    // 定义一个公钥 `ray_fee_pubkey`，这是 Raydium V4 的一个费用地址。
    // 我们将监控所有与这个地址相关的交易。7YttLkHDoNj9wyDur5pM1ejNaAvT9X4eqaYcHQqtj2G5是Raydium V4 的程序 ID (Program ID)
    let ray_fee_pubkey = Pubkey::from_str("7YttLkHDoNj9wyDur5pM1ejNaAvT9X4eqaYcHQqtj2G5")?;

    // 创建一个日志过滤器 `filters`。
    // `RpcTransactionLogsFilter::Mentions` 表示我们只对提及（mention）了 `ray_fee_pubkey` 的交易感兴趣。
    let filters = RpcTransactionLogsFilter::Mentions(vec![ray_fee_pubkey.to_string()]);
    // 创建日志订阅的配置 `config`。
    let config = RpcTransactionLogsConfig {
        // 设置确认级别为 `confirmed`，表示我们只关心已经被节点确认的交易。
        commitment: Some(CommitmentConfig::confirmed()), // 设置为 confirmed 确认级别
    };
    // 使用过滤器和配置来订阅交易日志。
    // `logs_subscribe` 返回一个元组：一个日志流 `logs_stream` 和一个取消订阅的函数 `unsubscriber`。
    //logs_stream的类型是Pin<Box<dyn Stream<Item = Response<RpcLogsResponse>> + Send>>
    //1.Pin是Rust中的一个trait，用于将类型固定在内存中的某个位置，防止被移动
    //2.Box<dyn Stream>代表了动态分发的Stream流trait对象，dyn Stream代表任何实现了stream的trait
    //3.异步编程中的许多操作是跨线程的，send确保stream trait 确保类型在多个线程之间安全传递
    let (mut logs_stream, unsubscriber) = ps_client.logs_subscribe(filters, config).await?;
    println!("开始监控..."); // 输出提示信息，表示开始监控。

    // 使用 `while let` 循环来异步处理从 `logs_stream` 中接收到的每一条日志。
    // `.next().await` 会等待下一条日志的到来。
    while let Some(response) = logs_stream.next().await {
        // 从响应中获取 `value`，其中包含了日志的具体信息。
        let value = response.value;
        // 获取交易的签名。
        let signature = value.signature;
        // 将签名转换为字符串格式。
        let transaction_signature_str = signature.as_str();

        // 将字符串格式的签名解析回 `Signature` 类型，以便后续查询使用。
        let transaction_signature = Signature::from_str(transaction_signature_str)
            .map_err(|e| format!("签名错误: {}", e))?;

        // 为 `get_transaction` RPC 调用创建配置。
        let config = RpcTransactionConfig {
            // UiTransactionEncoding 是一个枚举，通常定义了几种不同的编码格式，用来表示如何编码交易信息，指定返回的交易数据使用 JSON 编码。
            encoding: Some(UiTransactionEncoding::Json),
            // 指定我们期望的最高交易版本号，0 表示 Legacy Transaction。
            max_supported_transaction_version: Some(0), // 这里指定支持交易版本 0
            // 同样设置确认级别为 `confirmed`。
            commitment: Some(CommitmentConfig::confirmed()),
        };

        // 使用交易签名和配置，通过 RPC 客户端获取完整的交易详情。
        let transaction = rpc_client.get_transaction_with_config(&transaction_signature, config);
        // `transaction?` 会在获取失败时提前返回错误。
        // `.transaction.meta.map` 用于安全地访问交易的元数据（meta）。
        transaction?.transaction.meta.map(|meta| {
            // `.post_token_balances.map` 用于安全地访问交易执行后的代币余额（post_token_balances）。
            meta.post_token_balances.map(|token_balances: Vec<_>| {
                // 遍历所有的代币余额变化。
                token_balances.iter().for_each(|token_balance| {
                    // 使用 `as_deref` 将 `Option<String>` 转换为 `Option<&str>`，然后与 `Some("...")` 比较。
                    // 这种方法更简洁，可以避免复杂的 `if let` 嵌套，并解决类型推断问题。
                    if matches!(&token_balance.owner, solana_transaction_status::option_serializer::OptionSerializer::Some(s) if s == "5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1")
                        && token_balance.mint != "So11111111111111111111111111111111111111112"
                    {
                        // 如果条件满足，就打印新代币的信息。
                        println!("{}", "========== 发现新币 ==========".bold().blue());
                        println!("代币地址: {}", token_balance.mint.green());
                        println!("{}", "=====================================".bold().blue());
                    }
                });
            });
        });
    }

    // 当循环结束时（虽然在这个例子中它会一直运行），调用 `unsubscriber` 函数来关闭 WebSocket 连接。
    unsubscriber().await;

    // 如果程序正常退出，返回 `Ok(())`。
    Ok(())
}
