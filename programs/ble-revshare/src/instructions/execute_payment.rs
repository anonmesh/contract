use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount as SplTokenAccount, Mint as SplMint, Transfer, transfer};
use arcium_anchor::prelude::*;

use super::payment_callback::PaymentV3Callback;
use crate::{ArciumSignerAccount, ID, ID_CONST};
use crate::constants::{COMP_DEF_OFFSET_PAYMENT_STATS, TREASURY_WALLET};
use crate::errors::ErrorCode;
use crate::events::PaymentEvent;
use crate::state::PaymentReceipt;

#[queue_computation_accounts("payment_v3", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64, _payment_id: [u8; 32])]
pub struct ExecutePayment<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = 8 + PaymentReceipt::INIT_SPACE,
        seeds = [b"payment_receipt", _payment_id.as_ref()],
        bump,
    )]
    pub payment_receipt: Account<'info, PaymentReceipt>,

    /// CHECK: Optional broadcaster signer
    pub broadcaster: Option<Signer<'info>>,

    /// CHECK: recipient — only used as token transfer destination
    pub recipient: UncheckedAccount<'info>,

    pub mint: Account<'info, SplMint>,

    #[account(mut)]
    pub payer_token_account: Box<Account<'info, SplTokenAccount>>,

    #[account(mut)]
    pub recipient_token_account: Box<Account<'info, SplTokenAccount>>,

    #[account(
        mut,
        constraint = treasury_token_account.owner == TREASURY_WALLET @ ErrorCode::InvalidTreasury
    )]
    pub treasury_token_account: Box<Account<'info, SplTokenAccount>>,

    #[account(mut)]
    pub broadcaster_token_account: Option<Box<Account<'info, SplTokenAccount>>>,

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

    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_PAYMENT_STATS))]
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

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub arcium_program: Program<'info, Arcium>,
}

pub(crate) fn handler(
    ctx: Context<ExecutePayment>,
    computation_offset: u64,
    _payment_id: [u8; 32],
    amount: u64,
    encrypted_amount: [u8; 32],
    nonce: u128,
    pub_key: [u8; 32],
    expires_at: i64,
) -> Result<()> {
    require!(
        Clock::get()?.unix_timestamp <= expires_at,
        ErrorCode::PaymentExpired
    );

    msg!("Payer: {}", ctx.accounts.payer.key());
    msg!("Sign PDA: {}", ctx.accounts.sign_pda_account.key());

    require_keys_eq!(
        ctx.accounts.payer_token_account.owner,
        ctx.accounts.payer.key(),
        ErrorCode::InvalidTokenAccount
    );
    require_keys_eq!(
        ctx.accounts.payer_token_account.mint,
        ctx.accounts.mint.key(),
        ErrorCode::InvalidTokenAccount
    );
    require_keys_eq!(
        ctx.accounts.recipient_token_account.owner,
        ctx.accounts.recipient.key(),
        ErrorCode::InvalidTokenAccount
    );
    require_keys_eq!(
        ctx.accounts.recipient_token_account.mint,
        ctx.accounts.mint.key(),
        ErrorCode::InvalidTokenAccount
    );
    require_keys_eq!(
        ctx.accounts.treasury_token_account.mint,
        ctx.accounts.mint.key(),
        ErrorCode::InvalidTokenAccount
    );

    let treasury_total_amount = amount
        .checked_mul(2)
        .ok_or(ErrorCode::MathOverflow)?
        / 100;

    let mut broadcaster_share_amount: u64 = 0;

    if let Some(broadcaster) = &ctx.accounts.broadcaster {
        msg!("Broadcaster: {}", broadcaster.key());

        let broadcaster_token_account = ctx
            .accounts
            .broadcaster_token_account
            .as_ref()
            .ok_or(ErrorCode::MissingBroadcasterAccount)?;

        require_keys_eq!(
            broadcaster_token_account.owner,
            broadcaster.key(),
            ErrorCode::MissingBroadcasterAccount
        );
        require_keys_eq!(
            broadcaster_token_account.mint,
            ctx.accounts.mint.key(),
            ErrorCode::MissingBroadcasterAccount
        );

        broadcaster_share_amount = treasury_total_amount
            .checked_mul(30)
            .ok_or(ErrorCode::MathOverflow)?
            / 100;
    } else if ctx.accounts.broadcaster_token_account.is_some() {
        return Err(ErrorCode::BroadcasterSignatureRequired.into());
    }

    let treasury_share_amount = treasury_total_amount
        .checked_sub(broadcaster_share_amount)
        .ok_or(ErrorCode::MathOverflow)?;

    let recipient_share_amount = amount
        .checked_sub(treasury_total_amount)
        .ok_or(ErrorCode::MathOverflow)?;

    transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.payer_token_account.to_account_info(),
                to: ctx.accounts.recipient_token_account.to_account_info(),
                authority: ctx.accounts.payer.to_account_info(),
            },
        ),
        recipient_share_amount,
    )?;

    if treasury_share_amount > 0 {
        transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.payer_token_account.to_account_info(),
                    to: ctx.accounts.treasury_token_account.to_account_info(),
                    authority: ctx.accounts.payer.to_account_info(),
                },
            ),
            treasury_share_amount,
        )?;
    }

    if broadcaster_share_amount > 0 {
        let broadcaster_token_account = ctx
            .accounts
            .broadcaster_token_account
            .as_ref()
            .ok_or(ErrorCode::MissingBroadcasterAccount)?;
        transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.payer_token_account.to_account_info(),
                    to: broadcaster_token_account.to_account_info(),
                    authority: ctx.accounts.payer.to_account_info(),
                },
            ),
            broadcaster_share_amount,
        )?;
    }

    let args = ArgBuilder::new()
        .x25519_pubkey(pub_key)
        .plaintext_u128(nonce)
        .encrypted_u64(encrypted_amount)
        .build();

    ctx.accounts.payment_receipt.bump = ctx.bumps.payment_receipt;
    ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;

    queue_computation(
        ctx.accounts,
        computation_offset,
        args,
        vec![PaymentV3Callback::callback_ix(
            computation_offset,
            &ctx.accounts.mxe_account,
            &[],
        )?],
        1,
        0,
    )?;

    emit!(PaymentEvent {
        payer: ctx.accounts.payer.key(),
        recipient: ctx.accounts.recipient.key(),
        broadcaster: ctx.accounts.broadcaster.as_ref().map(|b| b.key()),
        amount,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
