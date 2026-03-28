use anchor_lang::prelude::*;

#[event]
pub struct PaymentEvent {
    pub payer: Pubkey,
    pub recipient: Pubkey,
    pub broadcaster: Option<Pubkey>,
    pub amount: u64,
    pub timestamp: i64,
}
