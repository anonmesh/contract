use anchor_lang::prelude::*;

use crate::errors::ErrorCode;
use crate::events::BeaconRegistered;
use crate::state::BeaconRegistry;

#[derive(Accounts)]
pub struct RegisterBeacon<'info> {
    #[account(mut)]
    pub operator: Signer<'info>,

    #[account(
        init,
        payer = operator,
        space = 8 + BeaconRegistry::INIT_SPACE,
        seeds = [b"beacon", operator.key().as_ref()],
        bump,
    )]
    pub beacon: Account<'info, BeaconRegistry>,

    pub system_program: Program<'info, System>,
}

pub(crate) fn handler(
    ctx: Context<RegisterBeacon>,
    rns_dest_hash: [u8; 16],
    region_code: [u8; 4],
) -> Result<()> {
    require!(rns_dest_hash != [0u8; 16], ErrorCode::InvalidRnsHash);
    require!(
        region_code.iter().all(|b| (0x20..=0x7E).contains(b)),
        ErrorCode::InvalidRegionCode
    );

    let now = Clock::get()?.unix_timestamp;
    let beacon = &mut ctx.accounts.beacon;
    let operator = ctx.accounts.operator.key();

    beacon.bump = ctx.bumps.beacon;
    beacon.operator = operator;
    beacon.rns_dest_hash = rns_dest_hash;
    beacon.region_code = region_code;
    beacon.registered_at = now;
    beacon.last_heartbeat = now;
    beacon.heartbeat_count = 1;

    emit!(BeaconRegistered {
        operator,
        beacon_pda: beacon.key(),
        rns_dest_hash,
        region_code,
        registered_at: now,
    });

    msg!(
        "beacon registered: operator={} region={:?}",
        operator,
        region_code
    );

    Ok(())
}
