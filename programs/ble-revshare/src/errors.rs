use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("The broadcaster signature is required when providing a broadcaster account.")]
    BroadcasterSignatureRequired,
    #[msg("Broadcaster token account is missing but broadcaster is present.")]
    MissingBroadcasterAccount,
    #[msg("The computation was aborted")]
    AbortedComputation,
    #[msg("The cluster is not set")]
    ClusterNotSet,
    #[msg("A token account had an unexpected owner or mint")]
    InvalidTokenAccount,
    #[msg("Arithmetic overflow while computing payment shares")]
    MathOverflow,
    #[msg("The treasury token account does not belong to the expected treasury wallet")]
    InvalidTreasury,
    #[msg("Payment payload has expired")]
    PaymentExpired,
}
