use anchor_lang::prelude::*;

use crate::errors::ErrorCode;
use crate::events::RelayStatsInitialized;
use crate::state::{BeaconRegistry, PrivateRelayStats};

/// One-shot initialization of an operator's encrypted relay counter.
///
/// The operator supplies:
/// - `initial_ciphertext`: client-side encryption of `RelayCount { count: 0 }`
///   under their own x25519 Shared keypair, using the Arcium client SDK.
/// - `initial_nonce`: the nonce used during that encryption.
/// - `x25519_pubkey`: the operator's x25519 public key (stored on-chain so
///   the operator can re-derive the decrypt path on any device).
///
/// The contract makes no claim about the plaintext value — it just persists
/// the ciphertext verbatim. Convention: starting plaintext is 0. An operator
/// that submits a non-zero initial value is only fooling themselves.
#[derive(Accounts)]
pub struct InitRelayStats<'info> {
    #[account(mut)]
    pub operator: Signer<'info>,

    /// Existing BeaconRegistry PDA — proves operator has a registered beacon.
    /// `has_one = operator` ensures we can't pair stats with someone else's
    /// beacon.
    #[account(
        seeds = [b"beacon", operator.key().as_ref()],
        bump = beacon.bump,
        has_one = operator @ ErrorCode::OperatorMismatch,
    )]
    pub beacon: Account<'info, BeaconRegistry>,

    #[account(
        init,
        payer = operator,
        space = 8 + PrivateRelayStats::INIT_SPACE,
        seeds = [b"relay_stats", operator.key().as_ref()],
        bump,
    )]
    pub stats: Account<'info, PrivateRelayStats>,

    pub system_program: Program<'info, System>,
}

pub(crate) fn handler(
    ctx: Context<InitRelayStats>,
    initial_ciphertext: [u8; 32],
    initial_nonce: u128,
    x25519_pubkey: [u8; 32],
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let stats = &mut ctx.accounts.stats;
    let operator = ctx.accounts.operator.key();

    stats.bump = ctx.bumps.stats;
    stats.operator = operator;
    stats.beacon_pda = ctx.accounts.beacon.key();
    stats.encrypted_count = initial_ciphertext;
    stats.nonce = initial_nonce;
    stats.x25519_pubkey = x25519_pubkey;
    stats.last_updated = now;
    stats.update_count = 0;

    emit!(RelayStatsInitialized {
        operator,
        stats_pda: stats.key(),
        beacon_pda: ctx.accounts.beacon.key(),
        timestamp: now,
    });

    msg!("relay stats initialized: operator={}", operator);
    Ok(())
}
