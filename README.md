# BLE Revenue Sharing

A Solana smart contract for handling revenue sharing payments with encrypted computation support via Arcium.

## Overview

This program implements a payment distribution system where payments can be split between recipients and broadcasters. It uses Arcium's confidential computing infrastructure for encrypted payment processing.

## Architecture

### Programs

- `ble-revshare`: Main program handling payment execution and revenue distribution

### Key Features

- Token whitelist management
- Payment execution with broadcaster revenue sharing (70/30 split)
- Arcium encrypted computation integration
- Payment statistics tracking via confidential computations

## Prerequisites

- Rust 1.79.0 or later
- Node.js 18+ and Yarn
- Solana CLI
- Arcium CLI

## Installation

```bash
# Install dependencies
yarn install

# Build the program
arcium build
```

## Deployment

### 1. Build the program

```bash
arcium build
```

### 2. Deploy to devnet

```bash
export ANCHOR_PROVIDER_URL="https://api.devnet.solana.com"
export ANCHOR_WALLET="~/.config/solana/id.json"
export ARCIUM_CLUSTER_OFFSET=456

arcium deploy \
  --cluster-offset 456 \
  --keypair-path ~/.config/solana/id.json \
  --recovery-set-size 4 \
  --rpc-url "https://devnet.helius-rpc.com/?api-key=YOUR_API_KEY"
```

### 3. Initialize computation definition

```bash
npx ts-node init-comp-def-final.ts
```

### 4. Whitelist tokens

```bash
npx ts-node scripts/whitelist-tokens.ts
```

## Testing

```bash
export ANCHOR_PROVIDER_URL="https://devnet.helius-rpc.com/?api-key=YOUR_API_KEY"
export ANCHOR_WALLET="~/.config/solana/id.json"
export ARCIUM_CLUSTER_OFFSET=456

yarn test
```

## Project Structure

```
.
├── programs/
│   └── ble-revshare/
│       └── src/
│           └── lib.rs          # Main program logic
├── tests/
│   └── ble-revshare.ts         # Test suite
├── encrypted-ixs/              # Encrypted instruction artifacts
├── migrations/                 # Deployment migrations
└── Arcium.toml                 # Arcium configuration
```

## Environment Variables

| Variable | Description | Required |
|----------|-------------|----------|
| `ANCHOR_PROVIDER_URL` | Solana RPC URL | Yes |
| `ANCHOR_WALLET` | Path to wallet keypair | Yes |
| `ARCIUM_CLUSTER_OFFSET` | Arcium cluster offset | Yes |

## Program IDs

- Program ID: `7fvHNYVuZP6EYt68GLUa4kU8f8dCBSaGafL9aDhhtMZN`
- Arcium Program ID: `Arcj82pX7HxYKLR92qvgZUAd7vGS1k4hQvAFcPATFdEQ`

## License

MIT
