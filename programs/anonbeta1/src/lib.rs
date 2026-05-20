use anchor_lang::prelude::*;
use arcium_anchor::prelude::*;

pub mod constants;
pub mod errors;
pub mod events;
pub mod instructions;
pub mod state;

pub use errors::ErrorCode;
pub use instructions::beacon_bind_callback::*;
pub use instructions::execute_cosigned_transfer::*;
pub use instructions::init_beacon_bind_comp_def::*;
pub use instructions::init_relay_increment_comp_def::*;
pub use instructions::init_relay_stats::*;
pub use instructions::record_relay::*;
pub use instructions::register_beacon_private::*;
pub use instructions::relay_increment_callback::*;

declare_id!("anon7uu8UtVoFgS8GCSfw2RqyphJhkN3xEjgPwznYDe");

#[arcium_program]
pub mod anonbeta1 {
    use super::*;

    pub fn init_beacon_bind_comp_def(ctx: Context<InitBeaconBindCompDef>) -> Result<()> {
        instructions::init_beacon_bind_comp_def::handler(ctx)
    }

    pub fn init_relay_increment_comp_def(ctx: Context<InitRelayIncrementCompDef>) -> Result<()> {
        instructions::init_relay_increment_comp_def::handler(ctx)
    }

    pub fn execute_cosigned_transfer(
        ctx: Context<ExecuteCosignedTransfer>,
        settlement_id: [u8; 32],
        amount: u64,
        beacon_share_bps: u16,
    ) -> Result<()> {
        instructions::execute_cosigned_transfer::handler(
            ctx,
            settlement_id,
            amount,
            beacon_share_bps,
        )
    }

    pub fn register_beacon_private(
        ctx: Context<RegisterBeaconPrivate>,
        computation_offset: u64,
        encrypted_rns_dest_hash: [u8; 32],
        encrypted_region_code: [u8; 32],
        nonce: u128,
        pub_key: [u8; 32],
        region_code: [u8; 4],
        capabilities_bitmap: u32,
    ) -> Result<()> {
        instructions::register_beacon_private::handler(
            ctx,
            computation_offset,
            encrypted_rns_dest_hash,
            encrypted_region_code,
            nonce,
            pub_key,
            region_code,
            capabilities_bitmap,
        )
    }

    #[arcium_callback(encrypted_ix = "beacon_bind")]
    pub fn beacon_bind_callback(
        ctx: Context<BeaconBindCallback>,
        output: SignedComputationOutputs<BeaconBindOutput>,
    ) -> Result<()> {
        instructions::beacon_bind_callback::handler(ctx, output)
    }

    pub fn init_relay_stats(
        ctx: Context<InitRelayStats>,
        initial_ciphertext: [u8; 32],
        initial_nonce: u128,
        x25519_pubkey: [u8; 32],
    ) -> Result<()> {
        instructions::init_relay_stats::handler(
            ctx,
            initial_ciphertext,
            initial_nonce,
            x25519_pubkey,
        )
    }

    pub fn record_relay(
        ctx: Context<RecordRelay>,
        computation_offset: u64,
        relay_event_hash: [u8; 32],
        pub_key: [u8; 32],
    ) -> Result<()> {
        instructions::record_relay::handler(ctx, computation_offset, relay_event_hash, pub_key)
    }

    #[arcium_callback(encrypted_ix = "relay_increment")]
    pub fn relay_increment_callback(
        ctx: Context<RelayIncrementCallback>,
        output: SignedComputationOutputs<RelayIncrementOutput>,
    ) -> Result<()> {
        instructions::relay_increment_callback::handler(ctx, output)
    }
}
