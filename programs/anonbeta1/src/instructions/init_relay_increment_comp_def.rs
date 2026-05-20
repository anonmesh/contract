use anchor_lang::prelude::*;
use arcium_anchor::prelude::*;
use arcium_client::idl::arcium::types::{CircuitSource, OffChainCircuitSource};
use arcium_macros::circuit_hash;

use crate::ID;

#[init_computation_definition_accounts("relay_increment", payer)]
#[derive(Accounts)]
pub struct InitRelayIncrementCompDef<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(mut, address = derive_mxe_pda!())]
    pub mxe_account: Box<Account<'info, MXEAccount>>,

    #[account(mut)]
    /// CHECK: comp_def_account is derived and validated by the Arcium program.
    pub comp_def_account: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: address_lookup_table is validated by the Arcium program.
    pub address_lookup_table: UncheckedAccount<'info>,

    /// CHECK: LUT program account.
    pub lut_program: UncheckedAccount<'info>,

    pub arcium_program: Program<'info, Arcium>,
    pub system_program: Program<'info, System>,
}

pub(crate) fn handler(ctx: Context<InitRelayIncrementCompDef>) -> Result<()> {
    init_comp_def(
        ctx.accounts,
        Some(CircuitSource::OffChain(OffChainCircuitSource {
            // TODO(security): arcium-comp-def-hosting-durability. circuit_hash!
            // prevents tampering, but this Supabase bucket is a single
            // availability dependency for fresh MXE/cluster initialization.
            // Mirror to a durable commit-pinned or content-addressed host.
            source: "https://fosjbclmsqobydunswin.supabase.co/storage/v1/object/public/arcium/relay_increment.arcis".to_string(),
            hash: circuit_hash!("relay_increment"),
        })),
        None,
    )?;
    Ok(())
}
