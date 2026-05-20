# Anonmesh Contract

Solana programs and Arcium circuits for anonmesh beacon registration, co-signed
settlement, and private relay accounting.

## Overview

`anonbeta1` binds an operator wallet to an encrypted RNS transport identity,
then lets mobile clients send partially signed transfer transactions over
anonmesh/RNS. The announced beacon co-signs the transaction, submits it to
Solana, and receives its revenue share inside the same on-chain settlement.
Arcium is used for private operator-to-RNS binding and private relay stats.

## Architecture

### Programs

- `anonbeta1`: RNS beacon registry, co-signed token settlement, and
  Arcium-private operator-to-RNS binding / relay stats

### Key Features

- Private operator-to-RNS binding through the `beacon_bind` Arcium circuit
- Co-signed SPL-token settlement: sender signs first, beacon co-signs and earns
  the configured share
- Encrypted relay count through the `relay_increment` Arcium circuit
- Public state limited to beacon metadata, binding verification state,
  settlement receipts, and relay liveness timestamps

## Mobile Flow

1. The beacon announces itself over anonmesh/RNS.
2. The mobile app creates a transfer with the beacon operator as a required
   co-signer, fills `{ settlement_id, recipient ATA, amount, beacon_share_bps }`,
   and partially signs as the sender.
3. The partial transaction is sent over anonmesh/RNS to the beacon.
4. The beacon co-signs and submits `execute_cosigned_transfer`.
5. `anonbeta1` verifies the beacon is registered and Arcium-bound, transfers
   funds to the recipient and beacon ATA, and writes a settlement receipt.
6. The beacon can call `record_relay` with the settlement hash to update its
   encrypted Arcium relay counter.

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

### 3. Initialize computation definitions

```bash
# Initialize beacon_bind and relay_increment definitions after deploy.
# Use generated anonbeta1 client code; standalone init scripts are not included.
```

## Verification

```bash
export ANCHOR_PROVIDER_URL="https://devnet.helius-rpc.com/?api-key=YOUR_API_KEY"
export ANCHOR_WALLET="~/.config/solana/id.json"
export ARCIUM_CLUSTER_OFFSET=456

arcium build
```

`arcium build` is the canonical verification command for this repo. It compiles
the encrypted instructions and the Arcium-enabled Anchor program together.
Anchor-only builds are not sufficient for audit or release.

## Project Structure

```
.
├── programs/
│   └── anonbeta1/
│       └── src/
│           └── lib.rs          # Main program logic
└── encrypted-ixs/              # Encrypted instruction artifacts
```

## Environment Variables

| Variable | Description | Required |
|----------|-------------|----------|
| `ANCHOR_PROVIDER_URL` | Solana RPC URL | Yes |
| `ANCHOR_WALLET` | Path to wallet keypair | Yes |
| `ARCIUM_CLUSTER_OFFSET` | Arcium cluster offset | Yes |

The repository expects a local deploy keypair at the path you set in
`ANCHOR_WALLET` or pass with `--keypair-path`. Keypair files are intentionally
gitignored; fresh clones must provide their own.

## Program IDs

- anonbeta1 Program ID: `anon7uu8UtVoFgS8GCSfw2RqyphJhkN3xEjgPwznYDe`
- Arcium Program ID: `Arcj82pX7HxYKLR92qvgZUAd7vGS1k4hQvAFcPATFdEQ`

## License

MIT
