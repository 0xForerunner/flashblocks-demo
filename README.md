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
- Rate: 6 transactions per second

## Usage

```bash
# Build the project
cargo build

# Run the application
cargo run
```

The application will:
1. Connect to your local OP Stack chain
2. Submit transactions continuously at 6 tx/sec
3. Generate histogram plots as `histogram_plot_XXXX.png` files every 2 seconds
4. Print statistics every 20 seconds showing transaction counts and average confirmation times

## Output

- **Histogram plots**: Live updating PNG files showing latency distributions
- **Console output**: Periodic statistics about transaction confirmation times
- **Two histograms**: One for transactions with pending tag, one without

The histograms help visualize the performance difference between using the pending tag override vs standard receipt polling on your OP Stack chain.

## Requirements

- Rust (latest stable)
- A running OP Stack chain on localhost:8545
- Sufficient ETH in the configured account for gas fees