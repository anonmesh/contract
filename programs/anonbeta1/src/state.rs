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

