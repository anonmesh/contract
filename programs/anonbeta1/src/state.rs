use anchor_lang::prelude::*;

/// On-chain registration for a single anonmesh beacon.
///
/// PDA seeds: `[b"beacon", operator.key().as_ref()]`
/// One registration per operator wallet.
#[account]
#[derive(InitSpace)]
pub struct BeaconRegistry {
    pub bump: u8,
    pub operator: Pubkey,
    pub rns_dest_hash: [u8; 16],
    pub region_code: [u8; 4],
    pub registered_at: i64,
    pub last_heartbeat: i64,
    pub heartbeat_count: u64,
}

/// Operator-only encrypted relay counter, maintained via Arcium MPC.
///
/// PDA seeds: `[b"relay_stats", operator.key().as_ref()]`
/// One per operator wallet.
///
/// The counter is encrypted under the operator's x25519 keypair. Only the
/// operator can decrypt. On-chain observers (explorer, judges) can verify
/// liveness (`last_updated`) but cannot see the count itself.
#[account]
#[derive(InitSpace)]
pub struct PrivateRelayStats {
    /// PDA bump
    pub bump: u8,
    /// Solana wallet that owns this stats account.
    pub operator: Pubkey,
    /// Linked `BeaconRegistry` PDA — proves this stats account is paired
    /// with a registered beacon.
    pub beacon_pda: Pubkey,
    /// Current encrypted counter ciphertext (Arcium Shared encryption,
    /// 32-byte output of the `relay_increment` circuit).
    pub encrypted_count: [u8; 32],
    /// Nonce associated with `encrypted_count`. Rotates on every increment.
    pub nonce: u128,
    /// Operator's x25519 public key used as the shared-encryption owner.
    /// Stored on-chain so the operator can re-derive the decrypt path on any
    /// device once they have their private key locally.
    pub x25519_pubkey: [u8; 32],
    /// Unix timestamp of most recent successful increment callback.
    pub last_updated: i64,
    /// Total number of successful `record_relay` callbacks. NOT private —
    /// this is the visible "how many times has the counter moved" metric.
    /// Useful for explorer liveness, NOT for relay totals (those are encrypted).
    pub update_count: u64,
}

