use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount as SplTokenAccount, Mint as SplMint, Transfer, transfer};
use arcium_anchor::prelude::*;
use arcium_client::idl::arcium::types::{CircuitSource, OffChainCircuitSource};
use arcium_macros::circuit_hash;

declare_id!("7xeQNUggKc2e5q6AQxsFBLBkXGg2p54kSx11zVainMks"); // Default localnet ID, to be updated

const COMP_DEF_OFFSET_PAYMENT_STATS: u32 = comp_def_offset("payment_stats");

const TREASURY_WALLET: Pubkey = pubkey!("DqyfDvr7yG4d3mtW6AiXgbuVM7GZWqn4RVARFbJxwtFc");

#[arcium_program]
pub mod ble_revshare {
    use super::*;



    pub fn init_payment_stats_comp_def(ctx: Context<InitPaymentStatsCompDef>) -> Result<()> {
         init_comp_def(
            ctx.accounts,
            Some(CircuitSource::OffChain(OffChainCircuitSource {
                // Placeholder: Reusing a hash/source from the previous project for structure.
                // In production, upload a new circuit for "payment_stats".
                source: "https://izromwpjybfzjqbkstqo.supabase.co/storage/v1/object/public/nb/circuits/payment_stats.arcis".to_string(), // process_payment
                hash: circuit_hash!("payment_stats"),
            })),
            None,
        )?;
        Ok(())
    }

    pub fn init_whitelist_token(ctx: Context<InitWhitelistToken>) -> Result<()> {
        let whitelist_entry = &mut ctx.accounts.whitelist_entry;
        whitelist_entry.mint = ctx.accounts.mint.key();
        whitelist_entry.bump = ctx.bumps.whitelist_entry;
        Ok(())
    }

    pub fn remove_whitelist_token(_ctx: Context<RemoveWhitelistToken>) -> Result<()> {
        // Closing the account is handled by the `close` constraint in the context
        Ok(())
    }

    pub fn execute_payment(
        ctx: Context<ExecutePayment>,
        computation_offset: u64,
        amount: u64,
        encrypted_amount: [u8; 32],  // Rescue ciphertext of amount (for Arcium)
        nonce: u128,
        pub_key: [u8; 32],
    ) -> Result<()> {
        msg!("Payer: {}", ctx.accounts.payer.key());
        msg!("Sign PDA: {}", ctx.accounts.sign_pda_account.key());

        // Validate token account ownership and mint consistency for safe transfers.
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

            // Broadcaster gets 30% of the 2% treasury cut.
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

        // Execute token transfers before queueing Arcium computation.
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

        // Keep the deserialized bump field in sync so queue_computation signs with the right seeds.
        ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;

        queue_computation(
            ctx.accounts,
            computation_offset,
            args,
            vec![PaymentStatsCallback::callback_ix(
                computation_offset,
                &ctx.accounts.mxe_account,
                &[]
            )?],
            1, // Verify input count
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

    #[arcium_callback(encrypted_ix = "payment_stats")]
    pub fn payment_stats_callback(
        ctx: Context<PaymentStatsCallback>,
        output: SignedComputationOutputs<PaymentStatsOutput>,
    ) -> Result<()> {
        let _result = match output.verify_output(
            &ctx.accounts.cluster_account,
            &ctx.accounts.computation_account
        ) {
            Ok(PaymentStatsOutput { field_0 }) => field_0,
            Err(e) => {
                msg!("Computation verification failed: {}", e);
                return Err(ErrorCode::AbortedComputation.into())
            },
        };
        // In a real app, update some global stats account here with the result
        Ok(())
    }
}

/// Accounts

#[derive(Accounts)]
pub struct InitWhitelistToken<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    pub mint: Account<'info, SplMint>,
    #[account(
        init,
        payer = admin,
        space = 8 + 32 + 1,
        seeds = [b"whitelist", mint.key().as_ref()],
        bump
    )]
    pub whitelist_entry: Account<'info, WhitelistEntry>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RemoveWhitelistToken<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    pub mint: Account<'info, SplMint>,
    #[account(
        mut,
        close = admin,
        seeds = [b"whitelist", mint.key().as_ref()],
        bump = whitelist_entry.bump
    )]
    pub whitelist_entry: Account<'info, WhitelistEntry>,
}

#[account]
pub struct WhitelistEntry {
    pub mint: Pubkey,
    pub bump: u8,
}

#[queue_computation_accounts("payment_stats", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64)]
pub struct ExecutePayment<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    
    /// CHECK: Optional broadcaster signer
    pub broadcaster: Option<Signer<'info>>,

    /// The recipient
    /// CHECK: Not safe to check ownership here generally, but we just engage in token transfer
    pub recipient: UncheckedAccount<'info>,

    pub mint: Account<'info, SplMint>,

    #[account(
        seeds = [b"whitelist", mint.key().as_ref()],
        bump = whitelist_entry.bump
    )]
    pub whitelist_entry: Account<'info, WhitelistEntry>,

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

    #[account(
        address = derive_mxe_pda!()
    )]
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

    #[account(
        address = derive_comp_def_pda!(COMP_DEF_OFFSET_PAYMENT_STATS)
    )]
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

#[callback_accounts("payment_stats")]
#[derive(Accounts)]
pub struct PaymentStatsCallback<'info> {
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

#[init_computation_definition_accounts("payment_stats", payer)]
#[derive(Accounts)]
pub struct InitPaymentStatsCompDef<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(mut, address = derive_mxe_pda!())]
    pub mxe_account: Box<Account<'info, MXEAccount>>,

    #[account(mut)]
    /// CHECK: comp_def_account
    pub comp_def_account: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: address_lookup_table validated by arcium program
    pub address_lookup_table: UncheckedAccount<'info>,

    /// CHECK: lut_program
    pub lut_program: UncheckedAccount<'info>,

    pub arcium_program: Program<'info, Arcium>,
    pub system_program: Program<'info, System>,
}

/// Events & Errors

#[event]
pub struct PaymentEvent {
    pub payer: Pubkey,
    pub recipient: Pubkey,
    pub broadcaster: Option<Pubkey>,
    pub amount: u64,
    pub timestamp: i64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("The broadcaster signature is required when providing a broadcaster account.")]
    BroadcasterSignatureRequired,
    #[msg("Broadcaster token account is missing but broadcaster is present.")]
    MissingBroadcasterAccount,
    #[msg("The computation was aborted")]
    AbortedComputation,
    #[msg("The cluster is not set")]
    ClusterNotSet,
    #[msg("A token account had an unexpected owner or mint")]
    InvalidTokenAccount,
    #[msg("Arithmetic overflow while computing payment shares")]
    MathOverflow,
    #[msg("The treasury token account does not belong to the expected treasury wallet")]
    InvalidTreasury,
}
