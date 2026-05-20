use anchor_lang::prelude::*;

use crate::errors::ErrorCode;
use crate::events::RelayStatsInitialized;
use crate::state::{BeaconRegistry, PrivateBeaconBinding, PrivateRelayStats};

#[derive(Accounts)]
pub struct InitRelayStats<'info> {
    #[account(mut)]
    pub operator: Signer<'info>,

    #[account(
        seeds = [b"beacon", operator.key().as_ref()],
        bump = beacon.bump,
        has_one = operator @ ErrorCode::OperatorMismatch,
        constraint = beacon.binding_verified @ ErrorCode::BindingPdaMismatch,
    )]
    pub beacon: Account<'info, BeaconRegistry>,

    #[account(
        seeds = [b"private_beacon", operator.key().as_ref()],
        bump = private_binding.bump,
        has_one = operator @ ErrorCode::OperatorMismatch,
        constraint = private_binding.beacon_pda == beacon.key() @ ErrorCode::BindingPdaMismatch,
        constraint = beacon.private_binding == private_binding.key() @ ErrorCode::BindingPdaMismatch,
        constraint = private_binding.verified @ ErrorCode::BindingPdaMismatch,
    )]
    pub private_binding: Account<'info, PrivateBeaconBinding>,

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
    require!(x25519_pubkey != [0u8; 32], ErrorCode::InvalidX25519Pubkey);
    // TODO(security): anonbeta1-relay-stats-zero-init-circuit. This ciphertext
    // is operator-asserted; do not key reward or reputation math off it until a
    // dedicated Arcium zero-init circuit emits the starting counter.
    require!(
        initial_ciphertext != [0u8; 32],
        ErrorCode::InvalidCiphertext
    );
    require!(
        x25519_pubkey == ctx.accounts.private_binding.x25519_pubkey,
        ErrorCode::OperatorMismatch
    );

    let now = Clock::get()?.unix_timestamp;
    let operator = ctx.accounts.operator.key();
    let stats = &mut ctx.accounts.stats;

    stats.bump = ctx.bumps.stats;
    stats.operator = operator;
    stats.beacon_pda = ctx.accounts.beacon.key();
    stats.encrypted_count = initial_ciphertext;
    stats.nonce = initial_nonce;
    stats.x25519_pubkey = x25519_pubkey;
    stats.last_updated = now;
    stats.last_relay_hash = [0u8; 32];
    stats.pending_relay_hash = [0u8; 32];

    emit!(RelayStatsInitialized {
        operator,
        stats_pda: stats.key(),
        beacon_pda: ctx.accounts.beacon.key(),
        timestamp: now,
    });

    Ok(())
}
