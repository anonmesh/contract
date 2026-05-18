use anchor_lang::prelude::*;
use arcium_anchor::prelude::*;

use crate::constants::COMP_DEF_OFFSET_RELAY_INCREMENT;
use crate::errors::ErrorCode;
use crate::events::RelayRecorded;
use crate::state::PrivateRelayStats;
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

    /// Operator's PrivateRelayStats — passed through callback-extra accounts
    /// in `record_relay` so we can update the on-chain ciphertext here.
    #[account(mut)]
    pub stats: Account<'info, PrivateRelayStats>,
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

    // Persist the new ciphertext + nonce.
    stats.encrypted_count = result.field_0.ciphertexts[0];
    stats.nonce = result.field_0.nonce;
    stats.last_updated = now;
    stats.update_count = stats
        .update_count
        .checked_add(1)
        .ok_or(ErrorCode::UpdateOverflow)?;

    emit!(RelayRecorded {
        operator: stats.operator,
        stats_pda: stats.key(),
        update_count: stats.update_count,
        timestamp: now,
    });

    Ok(())
}
