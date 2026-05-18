use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Reticulum dest hash cannot be all zeros")]
    InvalidRnsHash,
    #[msg("Region code must be printable ASCII")]
    InvalidRegionCode,
    #[msg("Operator does not match registered beacon")]
    OperatorMismatch,
    #[msg("Heartbeat counter overflow")]
    HeartbeatOverflow,
    #[msg("Arcium cluster is not set")]
    ClusterNotSet,
    #[msg("Encrypted computation was aborted or verification failed")]
    AbortedComputation,
    #[msg("Update counter overflow")]
    UpdateOverflow,
    #[msg("Relay stats must be paired with the operator's BeaconRegistry PDA")]
    BeaconPdaMismatch,
}
