use anchor_lang::prelude::*;
use arcium_anchor::prelude::*;

pub mod constants;
pub mod errors;
pub mod events;
pub mod instructions;
pub mod state;

pub use errors::ErrorCode;
pub use instructions::execute_payment::*;
pub use instructions::init_comp_def::*;
pub use instructions::payment_callback::*;

declare_id!("7xeQNUggKc2e5q6AQxsFBLBkXGg2p54kSx11zVainMks");

#[arcium_program]
pub mod ble_revshare {
    use super::*;

    pub fn init_payment_stats_comp_def(ctx: Context<InitPaymentStatsCompDef>) -> Result<()> {
        instructions::init_comp_def::handler(ctx)
    }

    pub fn execute_payment(
        ctx: Context<ExecutePayment>,
        computation_offset: u64,
        _payment_id: [u8; 32],
        amount: u64,
        encrypted_amount: [u8; 32],
        nonce: u128,
        pub_key: [u8; 32],
        expires_at: i64,
    ) -> Result<()> {
        instructions::execute_payment::handler(
            ctx,
            computation_offset,
            _payment_id,
            amount,
            encrypted_amount,
            nonce,
            pub_key,
            expires_at,
        )
    }

    #[arcium_callback(encrypted_ix = "payment_v3")]
    pub fn payment_v3_callback(
        ctx: Context<PaymentV3Callback>,
        output: SignedComputationOutputs<PaymentV3Output>,
    ) -> Result<()> {
        instructions::payment_callback::handler(ctx, output)
    }
}
