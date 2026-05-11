use anchor_lang::prelude::*;

use crate::errors::ErrorCode;
use crate::events::BeaconHeartbeat;
use crate::state::BeaconRegistry;

#[derive(Accounts)]
pub struct Heartbeat<'info> {
    pub operator: Signer<'info>,

    #[account(
        mut,
        seeds = [b"beacon", operator.key().as_ref()],
        bump = beacon.bump,
        has_one = operator @ ErrorCode::OperatorMismatch,
    )]
    pub beacon: Account<'info, BeaconRegistry>,
}

pub(crate) fn handler(ctx: Context<Heartbeat>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let beacon = &mut ctx.accounts.beacon;

    // Clock-skew guard: silently skip the timestamp update if the chain clock
    // hasn't moved forward (devnet clocks can be flaky). Still bump the count.
    if now > beacon.last_heartbeat {
        beacon.last_heartbeat = now;
    }

    beacon.heartbeat_count = beacon
        .heartbeat_count
        .checked_add(1)
        .ok_or(ErrorCode::HeartbeatOverflow)?;

    emit!(BeaconHeartbeat {
        operator: beacon.operator,
        beacon_pda: beacon.key(),
        heartbeat_count: beacon.heartbeat_count,
        timestamp: beacon.last_heartbeat,
    });

    Ok(())
}
