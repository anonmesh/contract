use anchor_lang::prelude::*;
use arcium_anchor::prelude::*;

pub mod constants;
pub mod errors;
pub mod events;
pub mod instructions;
pub mod state;

pub use errors::ErrorCode;
pub use instructions::register_beacon::*;
pub use instructions::heartbeat::*;
pub use instructions::init_beacon_bind_comp_def::*;
pub use instructions::register_beacon_private::*;
pub use instructions::beacon_bind_callback::*;

declare_id!("anon7uu8UtVoFgS8GCSfw2RqyphJhkN3xEjgPwznYDe");

#[arcium_program]
pub mod anonbeta1 {
    use super::*;

    pub fn register_beacon(
        ctx: Context<RegisterBeacon>,
        rns_dest_hash: [u8; 16],
        region_code: [u8; 4],
    ) -> Result<()> {
        instructions::register_beacon::handler(ctx, rns_dest_hash, region_code)
    }

    pub fn heartbeat(ctx: Context<Heartbeat>) -> Result<()> {
        instructions::heartbeat::handler(ctx)
    }

    pub fn init_beacon_bind_comp_def(ctx: Context<InitBeaconBindCompDef>) -> Result<()> {
        instructions::init_beacon_bind_comp_def::handler(ctx)
    }

    pub fn register_beacon_private(
        ctx: Context<RegisterBeaconPrivate>,
        computation_offset: u64,
        binding_id: [u8; 32],
        encrypted_rns_dest_hash: [u8; 32],
        encrypted_region_code: [u8; 32],
        nonce: u128,
        pub_key: [u8; 32],
    ) -> Result<()> {
        instructions::register_beacon_private::handler(
            ctx,
            computation_offset,
            binding_id,
            encrypted_rns_dest_hash,
            encrypted_region_code,
            nonce,
            pub_key,
        )
    }

    #[arcium_callback(encrypted_ix = "beacon_bind")]
    pub fn beacon_bind_callback(
        ctx: Context<BeaconBindCallback>,
        output: SignedComputationOutputs<BeaconBindOutput>,
    ) -> Result<()> {
        instructions::beacon_bind_callback::handler(ctx, output)
    }
}
