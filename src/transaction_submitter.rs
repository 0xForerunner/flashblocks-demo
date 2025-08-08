use alloy::{
    network::Ethereum,
    primitives::TxHash,
    providers::{Provider, fillers::FillProvider},
    rpc::types::TransactionRequest,
    transports::http::{Http, Client},
};
use anyhow::Result;

use crate::Config;

pub type EthProvider = FillProvider<
    alloy::providers::fillers::JoinFill<
        alloy::providers::fillers::JoinFill<
            alloy::providers::Identity,
            alloy::providers::fillers::JoinFill<
                alloy::providers::fillers::GasFiller,
                alloy::providers::fillers::JoinFill<
                    alloy::providers::fillers::BlobGasFiller,
                    alloy::providers::fillers::JoinFill<
                        alloy::providers::fillers::NonceFiller,
                        alloy::providers::fillers::ChainIdFiller,
                    >,
                >,
            >,
        >,
        alloy::providers::fillers::WalletFiller<alloy::network::EthereumWallet>,
    >,
    alloy::providers::RootProvider<Http<Client>>,
    Http<Client>,
    Ethereum,
>;

#[derive(Clone)]
pub struct TransactionSubmitter {
    provider: EthProvider,
    config: Config,
}

impl TransactionSubmitter {
    pub fn new(provider: EthProvider, config: Config) -> Self {
        Self { provider, config }
    }

    pub async fn submit_transaction(&self) -> Result<TxHash> {
        let tx_request = TransactionRequest::default()
            .to(self.config.to_address)
            .value(self.config.value)
            .gas_limit(21000);

        let pending_tx = self.provider.send_transaction(tx_request).await?;
        Ok(*pending_tx.tx_hash())
    }
}