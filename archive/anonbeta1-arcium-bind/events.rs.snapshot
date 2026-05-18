use anchor_lang::prelude::*;

#[event]
pub struct BeaconRegistered {
    pub operator: Pubkey,
    pub beacon_pda: Pubkey,
    pub rns_dest_hash: [u8; 16],
    pub region_code: [u8; 4],
    pub registered_at: i64,
}

#[event]
pub struct BeaconHeartbeat {
    pub operator: Pubkey,
    pub beacon_pda: Pubkey,
    pub heartbeat_count: u64,
    pub timestamp: i64,
}

#[event]
pub struct BeaconBindCompleted {
    pub commitment_ciphertext: [u8; 32],
    pub commitment_nonce: u128,
}
