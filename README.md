# Flashblocks Demo

A Rust application that submits transactions to an OP Stack chain and measures transaction confirmation latencies with live histograms.

## Features

- Submits 6 transactions per second to your local OP Stack chain
- Alternates between using and not using the pending tag for receipt polling
- Generates live updating histogram plots showing latency differences
- Uses latest alloy-rs for Ethereum interactions

## Configuration

The application uses default configuration in `main.rs`:
- RPC URL: `http://localhost:8545`
- Private key: Hardhat's first test account
- Target address: Hardhat's second test account
- Transaction value: 0.001 ETH
- Rate: 6 transactions per second (with ±30% random jitter to avoid block correlation)

## Usage

```bash
# Build the project
cargo build

# Run the application
cargo run
```

The application will:
1. Connect to your local OP Stack chain
2. Submit transactions continuously at 6 tx/sec in the background
3. Display a live updating histogram window using a native GUI
4. Print statistics periodically showing transaction counts and average confirmation times

## Output

- **Live histogram window**: Native GUI window that updates in real-time at 30+ FPS
- **Side-by-side bars**: For each time bin, red bars (with pending tag) and blue bars (without pending tag) appear side-by-side
- **Interactive plot**: Zoom, pan, and hover over bars for detailed information
- **Real-time stats**: Live display of transaction counts and average confirmation times
- **Legend**: Color-coded legend showing which bars represent which polling method
- **Smooth updates**: Native rendering with immediate visual updates as new data arrives

The interactive histogram window helps visualize the performance difference between using the pending tag override vs standard receipt polling on your OP Stack chain. The window updates continuously in real-time as transactions are submitted and confirmed.

## Requirements

- Rust (latest stable)
- A running OP Stack chain on localhost:8545
- Sufficient ETH in the configured account for gas fees