use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount as SplTokenAccount, Mint as SplMint, Transfer, transfer};
use arcium_anchor::prelude::*;
use arcium_client::idl::arcium::types::{CircuitSource, OffChainCircuitSource};
use arcium_macros::circuit_hash;

// Type aliases for better readability
pub type TokenAccountInfo<'info> = Account<'info, SplTokenAccount>;
pub type MintInfo<'info> = Account<'info, SplMint>;

declare_id!("7fvHNYVuZP6EYt68GLUa4kU8f8dCBSaGafL9aDhhtMZN"); // Default localnet ID, to be updated

const COMP_DEF_OFFSET_PAYMENT_STATS: u32 = comp_def_offset("payment_stats");

// The Arcium signer PDA is derived on the Arcium program (not our program) with:
// seeds: [b"ArciumSignerAccount", program_id], program_id: ARCIUM_PROG_ID
// Computed address: nhy7kthZGJjV3yqbyPuSeo2KhNriia4DQrii8jW3KcC
const ARCIUM_SIGNER_PDA: Pubkey = Pubkey::new_from_array([
    0x0b, 0xb5, 0x75, 0x62, 0xb6, 0x09, 0xc1, 0x85,
    0xa5, 0x29, 0x7d, 0x15, 0x73, 0x5e, 0xbd, 0x66,
    0x85, 0x37, 0x06, 0x10, 0xae, 0xde, 0xff, 0xb8,
    0x81, 0x8d, 0xa5, 0xb3, 0xff, 0x48, 0xf9, 0x21,
]);

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
        nonce: u128,
        pub_key: [u8; 32],
    ) -> Result<()> {
        
        let payment_amount = amount;
        msg!("Payer: {}", ctx.accounts.payer.key());
        msg!("Sign PDA: {}", ctx.accounts.sign_pda_account.key());
        if let Some(broadcaster) = &ctx.accounts.broadcaster {
             msg!("Broadcaster: {}", broadcaster.key());
        }

        let broadcaster_share_amount: u64;
// ... (lines 54-106 skipped, assuming content matches)

        let args = ArgBuilder::new()
             .x25519_pubkey(pub_key)
             // We can pass other stats here as needed by the circuit
             // For now, just passing the amount as a placeholder for volume tracking
            .plaintext_u64(amount) 
            .build();

        // Initialize the bump in the sign_pda_account if it was just created
        // The bump is stored at offset 8 (after the 8-byte discriminator)
        const SIGNER_ACCOUNT_BUMP_OFFSET: usize = 8;
        let bump = ctx.bumps.sign_pda_account;
        let sign_pda_info = ctx.accounts.sign_pda_account.to_account_info();
        let mut data = sign_pda_info.try_borrow_mut_data()?;
        data[SIGNER_ACCOUNT_BUMP_OFFSET] = bump;
        drop(data);

        queue_computation(
            ctx.accounts,
            computation_offset,
            args,
            None,
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
}
