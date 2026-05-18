use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::token::{self, Token, TokenAccount as SplTokenAccount, Mint as SplMint, Transfer};
use arcium_anchor::prelude::*;

use super::beacon_bind_callback::BeaconBindCallback;
use crate::constants::{COMP_DEF_OFFSET_BEACON_BIND, FEE_BPS, TREASURY_WALLET};
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

    // --- Payment accounts (optional SPL, always SOL fallback) ---

    /// Treasury SOL destination. Must match TREASURY_WALLET.
    /// CHECK: validated against TREASURY_WALLET constant
    #[account(mut, address = TREASURY_WALLET @ ErrorCode::InvalidTreasury)]
    pub treasury: UncheckedAccount<'info>,

    /// SPL mint — pass if paying with SPL token (e.g. USDC). Omit for SOL.
    pub mint: Option<Account<'info, SplMint>>,

    /// Payer's token account — required if mint is provided.
    #[account(mut)]
    pub payer_token_account: Option<Account<'info, SplTokenAccount>>,

    /// Treasury's token account — required if mint is provided.
    #[account(mut)]
    pub treasury_token_account: Option<Account<'info, SplTokenAccount>>,

    pub token_program: Option<Program<'info, Token>>,

    // --- Arcium accounts ---

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
    amount: u64,
    encrypted_rns_dest_hash: [u8; 32],
    encrypted_region_code: [u8; 32],
    nonce: u128,
    pub_key: [u8; 32],
) -> Result<()> {
    require!(amount > 0, ErrorCode::FeeRequired);

    let fee = amount
        .checked_mul(FEE_BPS)
        .ok_or(ErrorCode::MathOverflow)?
        / 10_000;

    // --- Payment: SPL or SOL ---
    if let Some(mint) = &ctx.accounts.mint {
        let payer_ata = ctx.accounts.payer_token_account
            .as_ref()
            .ok_or(ErrorCode::InvalidTokenAccount)?;
        let treasury_ata = ctx.accounts.treasury_token_account
            .as_ref()
            .ok_or(ErrorCode::InvalidTreasury)?;
        let token_program = ctx.accounts.token_program
            .as_ref()
            .ok_or(ErrorCode::InvalidTokenAccount)?;

        require_keys_eq!(payer_ata.owner, ctx.accounts.payer.key(), ErrorCode::InvalidTokenAccount);
        require_keys_eq!(payer_ata.mint, mint.key(), ErrorCode::InvalidTokenAccount);
        require_keys_eq!(treasury_ata.owner, TREASURY_WALLET, ErrorCode::InvalidTreasury);
        require_keys_eq!(treasury_ata.mint, mint.key(), ErrorCode::InvalidTreasury);

        token::transfer(
            CpiContext::new(
                token_program.to_account_info(),
                Transfer {
                    from: payer_ata.to_account_info(),
                    to: treasury_ata.to_account_info(),
                    authority: ctx.accounts.payer.to_account_info(),
                },
            ),
            fee,
        )?;
    } else {
        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.payer.to_account_info(),
                    to: ctx.accounts.treasury.to_account_info(),
                },
            ),
            fee,
        )?;
    }

    // --- Arcium MPC ---
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
        "beacon bind queued: operator={} fee={}",
        ctx.accounts.payer.key(),
        fee
    );

    Ok(())
}
