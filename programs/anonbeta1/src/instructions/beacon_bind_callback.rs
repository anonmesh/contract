use anchor_lang::prelude::*;
use arcium_anchor::prelude::*;

use crate::constants::COMP_DEF_OFFSET_BEACON_BIND;
use crate::errors::ErrorCode;
use crate::events::BeaconBindCompleted;
use crate::{ID, ID_CONST};

#[callback_accounts("beacon_bind")]
#[derive(Accounts)]
pub struct BeaconBindCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,

    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_BEACON_BIND))]
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
}

pub(crate) fn handler(
    ctx: Context<BeaconBindCallback>,
    output: SignedComputationOutputs<BeaconBindOutput>,
) -> Result<()> {
    let _result = match output.verify_output(
        &ctx.accounts.cluster_account,
        &ctx.accounts.computation_account,
    ) {
        Ok(BeaconBindOutput { field_0 }) => {
            msg!("beacon bind commitment verified");
            emit!(BeaconBindCompleted {
                commitment_ciphertext: field_0.ciphertexts[0],
                commitment_nonce: field_0.nonce,
            });
            field_0
        }
        Err(e) => {
            msg!("beacon bind verification failed: {}", e);
            return Err(ErrorCode::AbortedComputation.into());
        }
    };
    Ok(())
}
