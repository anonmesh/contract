use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct PaymentReceipt {
    pub bump: u8,
}
