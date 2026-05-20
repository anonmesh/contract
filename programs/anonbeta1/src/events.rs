use anchor_lang::prelude::*;

#[event]
pub struct BeaconRegistered {
    pub operator: Pubkey,
    pub beacon_pda: Pubkey,
    pub private_binding: Pubkey,
    pub binding_id: [u8; 32],
    pub region_code: [u8; 4],
    pub capabilities_bitmap: u32,
    pub registered_at: i64,
}

#[event]
pub struct BeaconBindCompleted {
    pub operator: Pubkey,
    pub beacon_pda: Pubkey,
    pub private_binding: Pubkey,
    pub commitment_ciphertext: [u8; 32],
    pub commitment_nonce: u128,
    pub timestamp: i64,
}

#[event]
pub struct RelayStatsInitialized {
    pub operator: Pubkey,
    pub stats_pda: Pubkey,
    pub beacon_pda: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct RelayRecorded {
    pub operator: Pubkey,
    pub beacon_pda: Pubkey,
    pub stats_pda: Pubkey,
    pub relay_event_hash: [u8; 32],
    pub timestamp: i64,
}

#[event]
pub struct CosignedSettlementExecuted {
    pub settlement_id: [u8; 32],
    pub sender: Pubkey,
    pub recipient_token_account: Pubkey,
    pub beacon_operator: Pubkey,
    pub beacon_pda: Pubkey,
    pub mint: Pubkey,
    pub amount: u64,
    pub beacon_share_amount: u64,
    pub timestamp: i64,
}
