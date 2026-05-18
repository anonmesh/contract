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
pub use instructions::init_relay_increment_comp_def::*;
pub use instructions::init_relay_stats::*;
pub use instructions::record_relay::*;
pub use instructions::relay_increment_callback::*;

declare_id!("anon7uu8UtVoFgS8GCSfw2RqyphJhkN3xEjgPwznYDe");

#[arcium_program]
pub mod anonbeta1 {
    use super::*;

    // ── v1: public registry ────────────────────────────────────────────

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

    // ── v2.A: encrypted relay counter via Arcium MPC ───────────────────

    /// One-time bootstrap by upgrade authority. Registers the
    /// `relay_increment` circuit on the MXE so subsequent `record_relay`
    /// calls can queue computations against it.
    pub fn init_relay_increment_comp_def(
        ctx: Context<InitRelayIncrementCompDef>,
    ) -> Result<()> {
        instructions::init_relay_increment_comp_def::handler(ctx)
    }

    /// Operator initializes their private encrypted counter PDA. Requires
    /// an existing `BeaconRegistry`. Client supplies an encrypted starting
    /// value (convention: encrypt(0)).
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

    /// Queue an Arcium computation that increments the operator's encrypted
    /// relay counter by 1. The new ciphertext is written by the callback.
    pub fn record_relay(
        ctx: Context<RecordRelay>,
        computation_offset: u64,
        new_nonce: u128,
        pub_key: [u8; 32],
    ) -> Result<()> {
        instructions::record_relay::handler(ctx, computation_offset, new_nonce, pub_key)
    }

    /// Arcium-invoked callback. Verifies the MPC output signature and
    /// persists the new ciphertext + nonce to the operator's `PrivateRelayStats`.
    #[arcium_callback(encrypted_ix = "relay_increment")]
    pub fn relay_increment_callback(
        ctx: Context<RelayIncrementCallback>,
        output: SignedComputationOutputs<RelayIncrementOutput>,
    ) -> Result<()> {
        instructions::relay_increment_callback::handler(ctx, output)
    }
}
