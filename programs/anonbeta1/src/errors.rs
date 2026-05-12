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
    #[msg("Cluster not set")]
    ClusterNotSet,
    #[msg("Computation was aborted or verification failed")]
    AbortedComputation,
    #[msg("Invalid token account")]
    InvalidTokenAccount,
    #[msg("Invalid treasury account")]
    InvalidTreasury,
    #[msg("Math overflow")]
    MathOverflow,
    #[msg("Registration fee required")]
    FeeRequired,
}
