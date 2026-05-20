use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Binding id cannot be all zeros")]
    InvalidBindingId,
    #[msg("Relay event hash cannot be all zeros")]
    InvalidRelayEventHash,
    #[msg("Settlement id cannot be all zeros")]
    InvalidSettlementId,
    #[msg("Encrypted ciphertext cannot be all zeros")]
    InvalidCiphertext,
    #[msg("x25519 public key cannot be all zeros")]
    InvalidX25519Pubkey,
    #[msg("Region code must be printable ASCII")]
    InvalidRegionCode,
    #[msg("Operator does not match registered beacon")]
    OperatorMismatch,
    #[msg("Private binding must match the operator's BeaconRegistry PDA")]
    BindingPdaMismatch,
    #[msg("Relay stats must be paired with the operator's BeaconRegistry PDA")]
    BeaconPdaMismatch,
    #[msg("A relay increment is already pending callback")]
    PendingRelayExists,
    #[msg("Relay event hash was already recorded")]
    DuplicateRelayEvent,
    #[msg("Token account owner or mint does not match expected accounts")]
    InvalidTokenAccount,
    #[msg("Beacon share basis points must be at most 10,000")]
    InvalidShareBps,
    #[msg("Arithmetic overflow")]
    MathOverflow,
    #[msg("Settlement counter overflow")]
    SettlementOverflow,
    #[msg("Private binding has already been verified")]
    BindingAlreadyVerified,
    #[msg("Arcium cluster is not set")]
    ClusterNotSet,
    #[msg("Encrypted computation was aborted or verification failed")]
    AbortedComputation,
}
