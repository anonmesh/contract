use anchor_lang::prelude::*;
use arcium_anchor::prelude::*;

use crate::constants::COMP_DEF_OFFSET_RELAY_INCREMENT;
use crate::errors::ErrorCode;
use crate::events::RelayRecorded;
use crate::state::{BeaconRegistry, PrivateRelayStats};
use crate::{ID, ID_CONST};

#[callback_accounts("relay_increment")]
#[derive(Accounts)]
pub struct RelayIncrementCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,

    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_RELAY_INCREMENT))]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,

    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Account<'info, MXEAccount>,

    /// CHECK: computation_account
    pub computation_account: UncheckedAccount<'info>,

    #[account(address = derive_cluster_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    pub cluster_account: Account<'info, Cluster>,

    #[account(address = ::anchor_lang::solana_program::sysvar::instructions::ID)]
    /// CHECK: instructions_sysvar
    pub instructions_sysvar: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [b"relay_stats", stats.operator.as_ref()],
        bump = stats.bump,
    )]
    pub stats: Account<'info, PrivateRelayStats>,

    #[account(
        mut,
        seeds = [b"beacon", stats.operator.as_ref()],
        bump = beacon.bump,
        constraint = beacon.key() == stats.beacon_pda @ ErrorCode::BeaconPdaMismatch,
        constraint = beacon.operator == stats.operator @ ErrorCode::OperatorMismatch,
    )]
    pub beacon: Account<'info, BeaconRegistry>,
}

pub(crate) fn handler(
    ctx: Context<RelayIncrementCallback>,
    output: SignedComputationOutputs<RelayIncrementOutput>,
) -> Result<()> {
    let result = match output.verify_output(
        &ctx.accounts.cluster_account,
        &ctx.accounts.computation_account,
    ) {
        Ok(r) => r,
        Err(e) => {
            msg!("relay increment verification failed: {}", e);
            return Err(ErrorCode::AbortedComputation.into());
        }
    };

    let now = Clock::get()?.unix_timestamp;
    let stats = &mut ctx.accounts.stats;
    let relay_event_hash = stats.pending_relay_hash;
    require!(
        relay_event_hash != [0u8; 32],
        ErrorCode::InvalidRelayEventHash
    );

    stats.encrypted_count = result.field_0.ciphertexts[0];
    stats.nonce = result.field_0.nonce;
    stats.last_updated = now;
    stats.last_relay_hash = relay_event_hash;
    stats.pending_relay_hash = [0u8; 32];

    let beacon = &mut ctx.accounts.beacon;
    beacon.last_relay_at = now;

    emit!(RelayRecorded {
        operator: stats.operator,
        beacon_pda: beacon.key(),
        stats_pda: stats.key(),
        relay_event_hash,
        timestamp: now,
    });

    Ok(())
}
