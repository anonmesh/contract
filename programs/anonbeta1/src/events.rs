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
pub struct RelayStatsInitialized {
    pub operator: Pubkey,
    pub stats_pda: Pubkey,
    pub beacon_pda: Pubkey,
    pub timestamp: i64,
}

/// Emitted by the Arcium callback after a successful encrypted counter
/// increment. The count itself is NOT in the event — only the public
/// liveness markers (which `stats_pda` moved, when, and how many total
/// updates the on-chain account has seen).
#[event]
pub struct RelayRecorded {
    pub operator: Pubkey,
    pub stats_pda: Pubkey,
    pub update_count: u64,
    pub timestamp: i64,
}

