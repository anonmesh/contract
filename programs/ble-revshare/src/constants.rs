use anchor_lang::prelude::*;
use arcium_anchor::prelude::comp_def_offset;

pub const COMP_DEF_OFFSET_PAYMENT_STATS: u32 = comp_def_offset("payment_v3");

pub const TREASURY_CUT_BPS: u64 = 2;
pub const BROADCASTER_SHARE_OF_TREASURY_BPS: u64 = 30;

pub const TREASURY_WALLET: Pubkey = pubkey!("DqyfDvr7yG4d3mtW6AiXgbuVM7GZWqn4RVARFbJxwtFc");
