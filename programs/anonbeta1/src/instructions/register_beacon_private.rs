use anchor_lang::prelude::*;
use arcium_anchor::prelude::*;

use super::beacon_bind_callback::BeaconBindCallback;
use crate::constants::COMP_DEF_OFFSET_BEACON_BIND;
use crate::errors::ErrorCode;
use crate::state::PrivateBeaconRegistry;
use crate::{ArciumSignerAccount, ID, ID_CONST};

#[queue_computation_accounts("beacon_bind", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64, _binding_id: [u8; 32])]
pub struct RegisterBeaconPrivate<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = 8 + PrivateBeaconRegistry::INIT_SPACE,
        seeds = [b"private_beacon", payer.key().as_ref()],
        bump,
    )]
    pub private_beacon: Account<'info, PrivateBeaconRegistry>,

    #[account(
        init_if_needed,
        space = 9,
        payer = payer,
        seeds = [b"ArciumSignerAccount"],
        bump,
    )]
    pub sign_pda_account: Account<'info, ArciumSignerAccount>,

    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Account<'info, MXEAccount>,

    #[account(
        mut,
        address = derive_mempool_pda!(mxe_account, ErrorCode::ClusterNotSet)
    )]
    /// CHECK: mempool_account
    pub mempool_account: UncheckedAccount<'info>,

    #[account(
        mut,
        address = derive_execpool_pda!(mxe_account, ErrorCode::ClusterNotSet)
    )]
    /// CHECK: executing_pool
    pub executing_pool: UncheckedAccount<'info>,

    #[account(
        mut,
        address = derive_comp_pda!(computation_offset, mxe_account, ErrorCode::ClusterNotSet)
    )]
    /// CHECK: computation_account
    pub computation_account: UncheckedAccount<'info>,

    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_BEACON_BIND))]
    pub comp_def_account: Box<Account<'info, ComputationDefinitionAccount>>,

    #[account(
        mut,
        address = derive_cluster_pda!(mxe_account, ErrorCode::ClusterNotSet)
    )]
    pub cluster_account: Box<Account<'info, Cluster>>,

    #[account(mut, address = ARCIUM_FEE_POOL_ACCOUNT_ADDRESS)]
    pub pool_account: Box<Account<'info, FeePool>>,

    #[account(mut, address = ARCIUM_CLOCK_ACCOUNT_ADDRESS)]
    pub clock_account: Box<Account<'info, ClockAccount>>,

    pub system_program: Program<'info, System>,
    pub arcium_program: Program<'info, Arcium>,
}

pub(crate) fn handler(
    ctx: Context<RegisterBeaconPrivate>,
    computation_offset: u64,
    _binding_id: [u8; 32],
    encrypted_rns_dest_hash: [u8; 32],
    encrypted_region_code: [u8; 32],
    nonce: u128,
    pub_key: [u8; 32],
) -> Result<()> {
    let args = ArgBuilder::new()
        .x25519_pubkey(pub_key)
        .plaintext_u128(nonce)
        .encrypted_u128(encrypted_rns_dest_hash)
        .encrypted_u32(encrypted_region_code)
        .build();

    let now = Clock::get()?.unix_timestamp;

    let private_beacon = &mut ctx.accounts.private_beacon;
    private_beacon.bump = ctx.bumps.private_beacon;
    private_beacon.operator = ctx.accounts.payer.key();
    private_beacon.x25519_pubkey = pub_key;
    private_beacon.verified = false;
    private_beacon.last_heartbeat = now;
    private_beacon.heartbeat_count = 0;

    ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;

    queue_computation(
        ctx.accounts,
        computation_offset,
        args,
        vec![BeaconBindCallback::callback_ix(
            computation_offset,
            &ctx.accounts.mxe_account,
            &[],
        )?],
        1,
        0,
    )?;

    msg!(
        "beacon bind queued: operator={}",
        ctx.accounts.payer.key()
    );

    Ok(())
}
