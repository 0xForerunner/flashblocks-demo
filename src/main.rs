use alloy::{
    network::EthereumWallet,
    primitives::{Address, U256},
    providers::ProviderBuilder,
    signers::local::PrivateKeySigner,
};
use anyhow::Result;
use std::{
    sync::{Arc, Mutex},
    time::Instant,
};
use tokio::time::interval;

mod transaction_submitter;
mod receipt_poller;
mod histogram_tracker;
mod visualizer;

use transaction_submitter::{TransactionSubmitter, EthProvider};
use receipt_poller::ReceiptPoller;
use histogram_tracker::HistogramTracker;
use visualizer::Visualizer;

#[derive(Clone)]
pub struct Config {
    pub rpc_url: String,
    pub private_key: String,
    pub to_address: Address,
    pub tx_per_second: u64,
    pub value: U256,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            rpc_url: "http://localhost:8545".to_string(),
            private_key: "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80".to_string(),
            to_address: "0x70997970C51812dc3A010C7d01b50e0d17dc79C8".parse().unwrap(),
            tx_per_second: 6,
            value: U256::from(1000000000000000u64), // 0.001 ETH
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::default();
    
    let signer: PrivateKeySigner = config.private_key.parse()?;
    let wallet = EthereumWallet::from(signer);
    
    let provider: EthProvider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_http(config.rpc_url.parse()?);
    
    let histogram_tracker = Arc::new(Mutex::new(HistogramTracker::new()));
    
    let submitter = TransactionSubmitter::new(provider.clone(), config.clone());
    let poller = ReceiptPoller::new(provider.clone());
    let visualizer = Visualizer::new(histogram_tracker.clone());
    
    tokio::spawn(async move {
        visualizer.run().await;
    });
    
    let mut interval = interval(tokio::time::Duration::from_millis(1000 / config.tx_per_second));
    let mut tx_counter = 0u64;
    
    loop {
        interval.tick().await;
        
        let use_pending = tx_counter % 2 == 0;
        let tx_hash = submitter.submit_transaction().await?;
        let start_time = Instant::now();
        
        let tracker = histogram_tracker.clone();
        let poller_clone = poller.clone();
        
        tokio::spawn(async move {
            if let Ok(_receipt) = poller_clone.wait_for_receipt(tx_hash, use_pending).await {
                let duration = start_time.elapsed();
                let mut tracker = tracker.lock().unwrap();
                tracker.record_transaction(duration, use_pending);
            }
        });
        
        tx_counter += 1;
    }
}
