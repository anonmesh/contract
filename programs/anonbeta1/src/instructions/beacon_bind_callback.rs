use anchor_lang::prelude::*;
use arcium_anchor::prelude::*;

use crate::constants::COMP_DEF_OFFSET_BEACON_BIND;
use crate::errors::ErrorCode;
use crate::events::BeaconBindCompleted;
use crate::state::{BeaconRegistry, PrivateBeaconBinding};
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

    #[account(
        mut,
        seeds = [b"private_beacon", private_binding.operator.as_ref()],
        bump = private_binding.bump,
    )]
    pub private_binding: Account<'info, PrivateBeaconBinding>,

    #[account(
        mut,
        seeds = [b"beacon", private_binding.operator.as_ref()],
        bump = beacon.bump,
        constraint = beacon.key() == private_binding.beacon_pda @ ErrorCode::BindingPdaMismatch,
        constraint = beacon.private_binding == private_binding.key() @ ErrorCode::BindingPdaMismatch,
        constraint = beacon.operator == private_binding.operator @ ErrorCode::OperatorMismatch,
    )]
    pub beacon: Account<'info, BeaconRegistry>,
}

pub(crate) fn handler(
    ctx: Context<BeaconBindCallback>,
    output: SignedComputationOutputs<BeaconBindOutput>,
) -> Result<()> {
    let result = match output.verify_output(
        &ctx.accounts.cluster_account,
        &ctx.accounts.computation_account,
    ) {
        Ok(r) => r,
        Err(e) => {
            msg!("beacon bind verification failed: {}", e);
            return Err(ErrorCode::AbortedComputation.into());
        }
    };

    let now = Clock::get()?.unix_timestamp;
    let private_binding = &mut ctx.accounts.private_binding;
    require!(!private_binding.verified, ErrorCode::BindingAlreadyVerified);

    private_binding.encrypted_commitment = result.field_0.ciphertexts[0];
    private_binding.commitment_nonce = result.field_0.nonce;
    private_binding.verified = true;
    private_binding.verified_at = now;

    let beacon = &mut ctx.accounts.beacon;
    beacon.binding_verified = true;
    beacon.binding_verified_at = now;

    emit!(BeaconBindCompleted {
        operator: private_binding.operator,
        beacon_pda: beacon.key(),
        private_binding: private_binding.key(),
        commitment_ciphertext: private_binding.encrypted_commitment,
        commitment_nonce: private_binding.commitment_nonce,
        timestamp: now,
    });

    Ok(())
}
