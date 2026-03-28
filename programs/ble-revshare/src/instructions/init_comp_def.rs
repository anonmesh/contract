use anchor_lang::prelude::*;
use arcium_anchor::prelude::*;
use arcium_client::idl::arcium::types::{CircuitSource, OffChainCircuitSource};
use arcium_macros::circuit_hash;

use crate::ID;

#[init_computation_definition_accounts("payment_v3", payer)]
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

pub(crate) fn handler(ctx: Context<InitPaymentStatsCompDef>) -> Result<()> {
    init_comp_def(
        ctx.accounts,
        Some(CircuitSource::OffChain(OffChainCircuitSource {
            source: "https://ofafszdpwzvtgnsjaarx.supabase.co/storage/v1/object/public/nb/circuits/payment_v3.arcis".to_string(),
            hash: circuit_hash!("payment_v3"),
        })),
        None,
    )?;
    Ok(())
}
