use anchor_lang::prelude::*;
use arcium_anchor::prelude::*;
use arcium_client::idl::arcium::types::{CircuitSource, OffChainCircuitSource};
use arcium_macros::circuit_hash;

use crate::ID;

/// One-time bootstrap to register the `relay_increment` Arcium circuit on the
/// MXE. Must be called by the upgrade authority once after deploy, before any
/// operator can call `record_relay`.
///
/// The circuit binary is hosted off-chain at the configured Supabase storage
/// URL. The hash is compiled in at build time via `circuit_hash!` and must
/// match the binary that gets uploaded.
#[init_computation_definition_accounts("relay_increment", payer)]
#[derive(Accounts)]
pub struct InitRelayIncrementCompDef<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(mut, address = derive_mxe_pda!())]
    pub mxe_account: Box<Account<'info, MXEAccount>>,

    #[account(mut)]
    /// CHECK: comp_def_account derived & validated by arcium program
    pub comp_def_account: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: address_lookup_table validated by arcium program
    pub address_lookup_table: UncheckedAccount<'info>,

    /// CHECK: lut_program
    pub lut_program: UncheckedAccount<'info>,

    pub arcium_program: Program<'info, Arcium>,
    pub system_program: Program<'info, System>,
}

pub(crate) fn handler(ctx: Context<InitRelayIncrementCompDef>) -> Result<()> {
    init_comp_def(
        ctx.accounts,
        Some(CircuitSource::OffChain(OffChainCircuitSource {
            // TODO(fawaz): upload build/relay_increment.arcis to this bucket
            // before invoking on devnet.
            source: "https://fosjbclmsqobydunswin.supabase.co/storage/v1/object/public/arcium/relay_increment.arcis".to_string(),
            hash: circuit_hash!("relay_increment"),
        })),
        None,
    )?;
    Ok(())
}
