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

/// Private beacon registration via Arcium MPC.
///
/// PDA seeds: `[b"private_beacon", operator.key().as_ref()]`
/// Stores the encrypted commitment from beacon_bind circuit.
#[account]
#[derive(InitSpace)]
pub struct PrivateBeaconRegistry {
    pub bump: u8,
    pub operator: Pubkey,
    pub ciphertext: [u8; 32],
    pub nonce: u128,
    pub x25519_pubkey: [u8; 32],
    pub commitment_hash: [u8; 32],
    pub verified: bool,
    pub last_heartbeat: i64,
    pub heartbeat_count: u64,
}
