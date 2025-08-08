use alloy::{
    network::EthereumWallet,
    primitives::{Address, U256},
    providers::ProviderBuilder,
    signers::local::PrivateKeySigner,
};
use anyhow::Result;
use rand::Rng;
use std::{
    sync::{Arc, Mutex},
    time::Instant,
};
use tokio::time::sleep;

mod histogram_tracker;
mod receipt_poller;
mod transaction_submitter;
mod visualizer;

use histogram_tracker::HistogramTracker;
use receipt_poller::ReceiptPoller;
use transaction_submitter::{EthProvider, TransactionSubmitter};
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
            private_key: "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
                .to_string(),
            to_address: "0x70997970C51812dc3A010C7d01b50e0d17dc79C8"
                .parse()
                .unwrap(),
            tx_per_second: 50,
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

    // Spawn transaction submission loop in background
    let tracker_clone = histogram_tracker.clone();
    let submit_handle = tokio::spawn(async move {
        let base_interval_ms = 1000 / config.tx_per_second;
        let mut tx_counter = 0u64;

        loop {
            // Add random jitter: ±30% of base interval to break correlation with block timing
            let jitter_range = (base_interval_ms as f64 * 0.3) as u64;
            let min_interval = base_interval_ms.saturating_sub(jitter_range);
            let max_interval = base_interval_ms + jitter_range;
            let random_interval = rand::thread_rng().gen_range(min_interval..=max_interval);

            sleep(tokio::time::Duration::from_millis(random_interval)).await;

            let use_pending = tx_counter % 2 == 0;
            match submitter.submit_transaction().await {
                Ok(tx_hash) => {
                    let start_time = Instant::now();
                    let tracker = tracker_clone.clone();
                    let poller_clone = poller.clone();

                    tokio::spawn(async move {
                        if let Ok(_receipt) =
                            poller_clone.wait_for_receipt(tx_hash, use_pending).await
                        {
                            let duration = start_time.elapsed();
                            let mut tracker = tracker.lock().unwrap();
                            tracker.record_transaction(duration, use_pending);
                        }
                    });
                }
                Err(e) => {
                    eprintln!("Failed to submit transaction: {}", e);
                }
            }

            tx_counter += 1;

            // Print jitter info occasionally for debugging
            if tx_counter % 30 == 0 {
                println!(
                    "TX #{}: Used {}ms interval (base: {}ms)",
                    tx_counter, random_interval, base_interval_ms
                );
            }
        }
    });

    // Run visualizer on main thread (required for GUI)
    println!("Starting live histogram window...");
    if let Err(e) = visualizer.run() {
        eprintln!("Visualizer error: {}", e);
    }

    // Clean shutdown
    submit_handle.abort();
    Ok(())
}
