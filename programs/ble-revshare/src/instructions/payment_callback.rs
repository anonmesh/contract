use anchor_lang::prelude::*;
use arcium_anchor::prelude::*;

use crate::{ID, ID_CONST};
use crate::constants::COMP_DEF_OFFSET_PAYMENT_STATS;
use crate::errors::ErrorCode;

#[callback_accounts("payment_v3")]
#[derive(Accounts)]
pub struct PaymentV3Callback<'info> {
    pub arcium_program: Program<'info, Arcium>,

    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_PAYMENT_STATS))]
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
    ctx: Context<PaymentV3Callback>,
    output: SignedComputationOutputs<PaymentV3Output>,
) -> Result<()> {
    let _result = match output.verify_output(
        &ctx.accounts.cluster_account,
        &ctx.accounts.computation_account,
    ) {
        Ok(PaymentV3Output { field_0 }) => field_0,
        Err(e) => {
            msg!("Computation verification failed: {}", e);
            return Err(ErrorCode::AbortedComputation.into());
        }
    };
    Ok(())
}
