use anchor_lang::prelude::*;

pub mod errors;
pub mod events;
pub mod instructions;
pub mod state;

pub use errors::ErrorCode;
pub use instructions::register_beacon::*;
pub use instructions::heartbeat::*;

declare_id!("anon7uu8UtVoFgS8GCSfw2RqyphJhkN3xEjgPwznYDe");

#[program]
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
}
