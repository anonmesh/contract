use anchor_lang::prelude::*;
use arcium_anchor::prelude::*;
use arcium_client::idl::arcium::types::CallbackAccount;

use super::relay_increment_callback::RelayIncrementCallback;
use crate::constants::COMP_DEF_OFFSET_RELAY_INCREMENT;
use crate::errors::ErrorCode;
use crate::state::{BeaconRegistry, PrivateRelayStats};
use crate::{ArciumSignerAccount, ID, ID_CONST};

#[queue_computation_accounts("relay_increment", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64)]
pub struct RecordRelay<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"beacon", payer.key().as_ref()],
        bump = beacon.bump,
        has_one = operator @ ErrorCode::OperatorMismatch,
        constraint = beacon.operator == payer.key() @ ErrorCode::OperatorMismatch,
        constraint = beacon.binding_verified @ ErrorCode::BindingPdaMismatch,
    )]
    pub beacon: Account<'info, BeaconRegistry>,

    /// CHECK: same as payer; constrained above.
    #[account(address = payer.key())]
    pub operator: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"relay_stats", payer.key().as_ref()],
        bump = stats.bump,
        has_one = operator @ ErrorCode::OperatorMismatch,
        constraint = stats.beacon_pda == beacon.key() @ ErrorCode::BeaconPdaMismatch,
    )]
    pub stats: Account<'info, PrivateRelayStats>,

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

    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_RELAY_INCREMENT))]
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
    ctx: Context<RecordRelay>,
    computation_offset: u64,
    relay_event_hash: [u8; 32],
    pub_key: [u8; 32],
) -> Result<()> {
    require!(
        relay_event_hash != [0u8; 32],
        ErrorCode::InvalidRelayEventHash
    );
    require!(
        ctx.accounts.stats.pending_relay_hash == [0u8; 32],
        ErrorCode::PendingRelayExists
    );
    // NOTE: this blocks immediate duplicate submissions only. Non-adjacent
    // replay detection remains a client/indexer responsibility in anonbeta1.
    require!(
        relay_event_hash != ctx.accounts.stats.last_relay_hash,
        ErrorCode::DuplicateRelayEvent
    );
    require!(
        ctx.accounts.stats.x25519_pubkey == pub_key,
        ErrorCode::OperatorMismatch
    );

    let args = ArgBuilder::new()
        .x25519_pubkey(pub_key)
        .plaintext_u128(ctx.accounts.stats.nonce)
        .encrypted_u64(ctx.accounts.stats.encrypted_count)
        .build();

    ctx.accounts.stats.pending_relay_hash = relay_event_hash;
    ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;

    let stats_acc = CallbackAccount {
        pubkey: ctx.accounts.stats.key(),
        is_writable: true,
    };
    let beacon_acc = CallbackAccount {
        pubkey: ctx.accounts.beacon.key(),
        is_writable: true,
    };

    queue_computation(
        ctx.accounts,
        computation_offset,
        args,
        vec![RelayIncrementCallback::callback_ix(
            computation_offset,
            &ctx.accounts.mxe_account,
            &[stats_acc, beacon_acc],
        )?],
        1,
        0,
    )?;

    Ok(())
}
