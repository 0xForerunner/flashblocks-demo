use alloy::{
    primitives::TxHash,
    providers::Provider,
    rpc::types::TransactionReceipt,
};
use anyhow::Result;
use tokio::time::{sleep, Duration};

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
        _use_pending: bool,
    ) -> Result<TransactionReceipt> {
        loop {
            let receipt = self.provider.get_transaction_receipt(tx_hash).await;

            match receipt {
                Ok(Some(receipt)) => {
                    if receipt.block_number.is_some() {
                        return Ok(receipt);
                    }
                }
                Ok(None) => {}
                Err(_) => {}
            }

            sleep(Duration::from_millis(100)).await;
        }
    }
}