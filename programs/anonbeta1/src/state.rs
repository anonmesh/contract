use anchor_lang::prelude::*;

/// Public registration for a single anonmesh RNS transport beacon.
///
/// PDA seeds: `[b"beacon", operator.key().as_ref()]`
/// One registration per operator wallet.
#[account]
#[derive(InitSpace)]
pub struct BeaconRegistry {
    pub bump: u8,
    pub operator: Pubkey,
    pub private_binding: Pubkey,
    pub binding_id: [u8; 32],
    /// TODO(security): anonbeta1-region-code-privacy-decision. This is
    /// intentionally public for explorer rendering. The encrypted region in the
    /// Arcium commitment binds this value to the private RNS destination for
    /// tamper-evidence; it is not intended to hide the operator's region. If the
    /// explorer no longer needs public region metadata, remove this field and
    /// the matching event field.
    pub region_code: [u8; 4],
    pub capabilities_bitmap: u32,
    pub registered_at: i64,
    pub binding_verified: bool,
    pub binding_verified_at: i64,
    pub last_relay_at: i64,
    /// Public settlement proof-of-work counter for explorer/indexer display.
    /// This intentionally differs from `PrivateRelayStats.encrypted_count`,
    /// which tracks operator-private relay throughput including traffic that
    /// does not settle through `execute_cosigned_transfer`.
    pub settlement_count: u64,
}

/// Arcium-verified private binding between an operator wallet and RNS
/// transport destination. The raw RNS destination hash stays encrypted.
///
/// PDA seeds: `[b"private_beacon", operator.key().as_ref()]`
#[account]
#[derive(InitSpace)]
pub struct PrivateBeaconBinding {
    pub bump: u8,
    pub operator: Pubkey,
    pub beacon_pda: Pubkey,
    pub binding_id: [u8; 32],
    pub encrypted_commitment: [u8; 32],
    pub commitment_nonce: u128,
    pub x25519_pubkey: [u8; 32],
    pub verified: bool,
    pub queued_at: i64,
    pub verified_at: i64,
}

/// Operator-only encrypted relay counter maintained via Arcium MPC.
///
/// PDA seeds: `[b"relay_stats", operator.key().as_ref()]`
#[account]
#[derive(InitSpace)]
pub struct PrivateRelayStats {
    pub bump: u8,
    pub operator: Pubkey,
    pub beacon_pda: Pubkey,
    pub encrypted_count: [u8; 32],
    pub nonce: u128,
    pub x25519_pubkey: [u8; 32],
    /// Private relay-throughput counter state. This may diverge from
    /// `BeaconRegistry.settlement_count` because not every relayed packet is a
    /// co-signed settlement.
    pub last_updated: i64,
    pub last_relay_hash: [u8; 32],
    pub pending_relay_hash: [u8; 32],
}

/// Public receipt for a co-signed settlement executed by a registered beacon.
///
/// Privacy scope: this account intentionally makes settlement activity public
/// as proof-of-work. Arcium hides the RNS transport identity and private relay
/// stats; it does not hide sender, destination token account, mint, amount, or
/// serving beacon for settled transfers.
///
/// PDA seeds: `[b"settlement", sender.key().as_ref(), settlement_id.as_ref()]`
/// NOTE: append-only by design; no close path is exposed for settlement
/// receipts.
#[account]
#[derive(InitSpace)]
pub struct CosignedSettlement {
    pub bump: u8,
    pub settlement_id: [u8; 32],
    pub sender: Pubkey,
    pub recipient_token_account: Pubkey,
    pub beacon_operator: Pubkey,
    pub beacon_pda: Pubkey,
    pub mint: Pubkey,
    pub amount: u64,
    pub beacon_share_amount: u64,
    pub executed_at: i64,
}
