use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Mint, Token, TokenAccount, Transfer};

use crate::errors::ErrorCode;
use crate::events::CosignedSettlementExecuted;
use crate::state::{BeaconRegistry, CosignedSettlement};

#[derive(Accounts)]
#[instruction(settlement_id: [u8; 32])]
pub struct ExecuteCosignedTransfer<'info> {
    #[account(mut)]
    pub sender: Signer<'info>,

    pub beacon_operator: Signer<'info>,

    #[account(
        mut,
        seeds = [b"beacon", beacon_operator.key().as_ref()],
        bump = beacon.bump,
        has_one = operator @ ErrorCode::OperatorMismatch,
        constraint = beacon.operator == beacon_operator.key() @ ErrorCode::OperatorMismatch,
        constraint = beacon.binding_verified @ ErrorCode::BindingPdaMismatch,
    )]
    pub beacon: Account<'info, BeaconRegistry>,

    /// CHECK: same as beacon_operator; constrained above for `has_one`.
    #[account(address = beacon_operator.key())]
    pub operator: UncheckedAccount<'info>,

    #[account(
        init,
        payer = sender,
        space = 8 + CosignedSettlement::INIT_SPACE,
        seeds = [b"settlement", sender.key().as_ref(), settlement_id.as_ref()],
        bump,
    )]
    pub settlement: Account<'info, CosignedSettlement>,

    pub mint: Account<'info, Mint>,

    #[account(mut)]
    pub sender_token_account: Account<'info, TokenAccount>,

    // NOTE: recipient ATA owner is intentionally unconstrained. The sender
    // chooses this destination in the signed transaction body.
    #[account(mut)]
    pub recipient_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub beacon_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub(crate) fn handler(
    ctx: Context<ExecuteCosignedTransfer>,
    settlement_id: [u8; 32],
    amount: u64,
    beacon_share_bps: u16,
) -> Result<()> {
    require!(settlement_id != [0u8; 32], ErrorCode::InvalidSettlementId);
    require!(beacon_share_bps <= 10_000, ErrorCode::InvalidShareBps);

    let mint = ctx.accounts.mint.key();
    require_keys_eq!(
        ctx.accounts.sender_token_account.owner,
        ctx.accounts.sender.key(),
        ErrorCode::InvalidTokenAccount
    );
    require_keys_eq!(
        ctx.accounts.sender_token_account.mint,
        mint,
        ErrorCode::InvalidTokenAccount
    );
    require_keys_eq!(
        ctx.accounts.recipient_token_account.mint,
        mint,
        ErrorCode::InvalidTokenAccount
    );
    require_keys_eq!(
        ctx.accounts.beacon_token_account.owner,
        ctx.accounts.beacon_operator.key(),
        ErrorCode::InvalidTokenAccount
    );
    require_keys_eq!(
        ctx.accounts.beacon_token_account.mint,
        mint,
        ErrorCode::InvalidTokenAccount
    );

    let beacon_share_amount = amount
        .checked_mul(beacon_share_bps as u64)
        .ok_or(ErrorCode::MathOverflow)?
        .checked_div(10_000)
        .ok_or(ErrorCode::MathOverflow)?;
    let recipient_amount = amount
        .checked_sub(beacon_share_amount)
        .ok_or(ErrorCode::MathOverflow)?;

    if recipient_amount > 0 {
        transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.sender_token_account.to_account_info(),
                    to: ctx.accounts.recipient_token_account.to_account_info(),
                    authority: ctx.accounts.sender.to_account_info(),
                },
            ),
            recipient_amount,
        )?;
    }

    if beacon_share_amount > 0 {
        transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.sender_token_account.to_account_info(),
                    to: ctx.accounts.beacon_token_account.to_account_info(),
                    authority: ctx.accounts.sender.to_account_info(),
                },
            ),
            beacon_share_amount,
        )?;
    }

    let now = Clock::get()?.unix_timestamp;
    let beacon = &mut ctx.accounts.beacon;
    beacon.last_relay_at = now;
    beacon.settlement_count = beacon
        .settlement_count
        .checked_add(1)
        .ok_or(ErrorCode::SettlementOverflow)?;

    let settlement = &mut ctx.accounts.settlement;
    settlement.bump = ctx.bumps.settlement;
    settlement.settlement_id = settlement_id;
    settlement.sender = ctx.accounts.sender.key();
    settlement.recipient_token_account = ctx.accounts.recipient_token_account.key();
    settlement.beacon_operator = ctx.accounts.beacon_operator.key();
    settlement.beacon_pda = beacon.key();
    settlement.mint = mint;
    settlement.amount = amount;
    settlement.beacon_share_amount = beacon_share_amount;
    settlement.executed_at = now;

    emit!(CosignedSettlementExecuted {
        settlement_id,
        sender: settlement.sender,
        recipient_token_account: settlement.recipient_token_account,
        beacon_operator: settlement.beacon_operator,
        beacon_pda: settlement.beacon_pda,
        mint,
        amount,
        beacon_share_amount,
        timestamp: now,
    });

    Ok(())
}
