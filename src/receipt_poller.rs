use alloy::{primitives::TxHash, providers::Provider, rpc::types::TransactionReceipt};
use anyhow::Result;
use serde_json::json;
use tokio::time::{Duration, sleep};

use crate::transaction_submitter::EthProvider;

#[derive(Clone)]
pub struct ReceiptPoller {
    provider: EthProvider,
}

impl ReceiptPoller {
    pub fn new(provider: EthProvider) -> Self {
        Self { provider }
    }

    pub async fn wait_for_receipt(
        &self,
        tx_hash: TxHash,
        use_pending: bool,
    ) -> Result<TransactionReceipt> {
        loop {
            let receipt = if use_pending {
                // Make raw RPC call with "pending" parameter
                self.get_receipt_with_pending(tx_hash).await
            } else {
                // Use normal provider method
                self.provider
                    .get_transaction_receipt(tx_hash)
                    .await
                    .map_err(|e| anyhow::anyhow!("Provider error: {}", e))
            };

            match receipt {
                Ok(Some(receipt)) => {
                    if receipt.block_number.is_some() {
                        return Ok(receipt);
                    }
                }
                Ok(None) => {}
                Err(_) => {}
            }

            sleep(Duration::from_millis(25)).await;
        }
    }

    async fn get_receipt_with_pending(
        &self,
        tx_hash: TxHash,
    ) -> Result<Option<TransactionReceipt>> {
        // Create parameters array with tx_hash and "pending"
        // let params = json!([tx_hash.to_string(), "pending"]);
        let params = json!([tx_hash.to_string()]);

        // Make raw RPC call
        let result: Result<Option<TransactionReceipt>, _> = self
            .provider
            .client()
            .request("eth_getTransactionReceipt", params)
            .await;

        match result {
            Ok(receipt) => Ok(receipt),
            Err(e) => Err(anyhow::anyhow!("RPC error: {}", e)),
        }
    }
}
