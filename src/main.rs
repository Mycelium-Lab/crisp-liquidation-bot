use near_crypto::InMemorySigner;
use near_jsonrpc_client::{methods, JsonRpcClient};
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use near_jsonrpc_primitives::types::transactions::TransactionInfo;
use near_primitives::transaction::{Action, FunctionCallAction, Transaction};
use near_primitives::types::BlockReference;
use near_primitives::views::FinalExecutionStatus;
use serde_json::json;
use tokio::time;

mod utils;

pub const CONTRACT_ID: &str = "dev-1674131081257-52430382583822";
pub const ACCOUNT_ID: &str = "abobac.testnet";
pub const SECRET_KEY: &str = "ed25519:4SSA3XVDM8Z8YaajAMQ8zFomDJbWNsuZc7gJmgAoKPphJHxbieUJ4Weieu6k8g5wDcybZTuGLwT83gcvoikdgSzo";
pub const VIEW_METHOD_NAME: &str = "get_liquidation_list";
pub const CALL_METHOD_NAME: &str = "liquidate";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let client = JsonRpcClient::connect("https://rpc.testnet.near.org");
    let signer_account_id = ACCOUNT_ID.parse()?;
    let signer_secret_key = SECRET_KEY.parse()?;
    let signer = near_crypto::InMemorySigner::from_secret_key(signer_account_id, signer_secret_key);
    loop {
        match run(client.clone(), signer.clone()).await {
            Ok(vec) => {
                println!("vec = {:?}", vec);
                if !vec.is_empty() {
                    match liquidate(client.clone(), signer.clone(), vec[0]).await {
                        Ok(_) => println!("Borrow {} has been liquidates", vec[0]),
                        _ => println!("Failure during liquidating borrow {}", vec[0]),
                    }
                }
            }
            _ => continue,
        }
    }
}

async fn run(
    client: JsonRpcClient,
    signer: InMemorySigner,
) -> Result<Vec<u128>, Box<dyn std::error::Error>> {
    let access_key_query_response = client
        .call(methods::query::RpcQueryRequest {
            block_reference: BlockReference::latest(),
            request: near_primitives::views::QueryRequest::ViewAccessKey {
                account_id: signer.account_id.clone(),
                public_key: signer.public_key.clone(),
            },
        })
        .await?;
    let current_nonce = match access_key_query_response.kind {
        QueryResponseKind::AccessKey(access_key) => access_key.nonce,
        _ => Err("failed to extract current nonce")?,
    };
    let transaction = Transaction {
        signer_id: signer.account_id.clone(),
        public_key: signer.public_key.clone(),
        nonce: current_nonce + 1,
        receiver_id: CONTRACT_ID.parse()?,
        block_hash: access_key_query_response.block_hash,
        actions: vec![Action::FunctionCall(FunctionCallAction {
            method_name: VIEW_METHOD_NAME.to_string(),
            args: json!({}).to_string().into_bytes(),
            gas: 100_000_000_000_000, // 100 TeraGas
            deposit: 0,
        })],
    };
    let request = methods::broadcast_tx_async::RpcBroadcastTxAsyncRequest {
        signed_transaction: transaction.sign(&signer),
    };
    let sent_at = time::Instant::now();
    let tx_hash = client.call(request).await?;
    loop {
        let response = client
            .call(methods::tx::RpcTransactionStatusRequest {
                transaction_info: TransactionInfo::TransactionId {
                    hash: tx_hash,
                    account_id: signer.account_id.clone(),
                },
            })
            .await;
        let received_at = time::Instant::now();
        let delta = (received_at - sent_at).as_secs();
        if delta > 60 {
            Err("time limit exceeded for the transaction to be recognized")?;
        }
        match response {
            Err(err) => match err.handler_error() {
                Some(methods::tx::RpcTransactionError::UnknownTransaction { .. }) => {
                    time::sleep(time::Duration::from_secs(2)).await;
                    continue;
                }
                _ => Err(err)?,
            },
            Ok(response) => {
                println!("response gotten after: {}s", delta);
                println!("response: {:#?}", response.status);
                match response.status {
                    FinalExecutionStatus::SuccessValue(value) => {
                        let vec: Vec<u128> = serde_json::from_slice(&value).unwrap();
                        return Ok(vec);
                    }
                    _ => Err("bad status")?,
                }
            }
        }
    }
}

async fn liquidate(
    client: JsonRpcClient,
    signer: InMemorySigner,
    id: u128,
) -> Result<(), Box<dyn std::error::Error>> {
    let access_key_query_response = client
        .call(methods::query::RpcQueryRequest {
            block_reference: BlockReference::latest(),
            request: near_primitives::views::QueryRequest::ViewAccessKey {
                account_id: signer.account_id.clone(),
                public_key: signer.public_key.clone(),
            },
        })
        .await?;
    let current_nonce = match access_key_query_response.kind {
        QueryResponseKind::AccessKey(access_key) => access_key.nonce,
        _ => Err("failed to extract current nonce")?,
    };
    let transaction = Transaction {
        signer_id: signer.account_id.clone(),
        public_key: signer.public_key.clone(),
        nonce: current_nonce + 1,
        receiver_id: CONTRACT_ID.parse()?,
        block_hash: access_key_query_response.block_hash,
        actions: vec![Action::FunctionCall(FunctionCallAction {
            method_name: CALL_METHOD_NAME.to_string(),
            args: json!({ "borrow_id": id }).to_string().into_bytes(),
            gas: 100_000_000_000_000, // 100 TeraGas
            deposit: 0,
        })],
    };
    let request = methods::broadcast_tx_async::RpcBroadcastTxAsyncRequest {
        signed_transaction: transaction.sign(&signer),
    };
    let sent_at = time::Instant::now();
    let tx_hash = client.call(request).await?;
    loop {
        let response = client
            .call(methods::tx::RpcTransactionStatusRequest {
                transaction_info: TransactionInfo::TransactionId {
                    hash: tx_hash,
                    account_id: signer.account_id.clone(),
                },
            })
            .await;
        let received_at = time::Instant::now();
        let delta = (received_at - sent_at).as_secs();
        if delta > 60 {
            Err("time limit exceeded for the transaction to be recognized")?;
        }
        match response {
            Err(err) => match err.handler_error() {
                Some(methods::tx::RpcTransactionError::UnknownTransaction { .. }) => {
                    time::sleep(time::Duration::from_secs(2)).await;
                    continue;
                }
                _ => Err(err)?,
            },
            Ok(_) => {
                println!("liquidate");
                println!("response gotten after: {}s", delta);
                // println!("response: {:#?}", response.status);
            }
        }
    }
}
