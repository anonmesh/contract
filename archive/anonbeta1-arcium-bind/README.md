# anonbeta1 — Arcium beacon-bind architecture (archived May 9, 2026)

Pre-Frontier `anonbeta1` had a private-registration path using Arcium MPC.
Stripped before Frontier submission to ship a registry-only program.
Preserved here as seed material for `anonbeta1` v2 (post-Frontier).

## What was archived

### Live instruction files (moved via `git mv`)
- `instructions/init_beacon_bind_comp_def.rs` — Arcium computation-definition initializer for the `beacon_bind` circuit
- `instructions/register_beacon_private.rs` — Public entry that queues an Arcium MPC computation binding an operator to an encrypted `(rns_dest_hash, region_code)` commitment, gated by an SOL/SPL registration fee
- `instructions/beacon_bind_callback.rs` — Arcium MPC callback handler that stores the commitment ciphertext into a `PrivateBeaconRegistry` PDA

### Snapshots (copies of the pre-strip files)
- `state.rs.snapshot` — included `PrivateBeaconRegistry` account
- `events.rs.snapshot` — included `BeaconBindCompleted` event
- `errors.rs.snapshot` — included Arcium/SPL/fee error variants
- `constants.rs.snapshot` — `COMP_DEF_OFFSET_BEACON_BIND`, `TREASURY_WALLET`, `FEE_BPS`
- `lib.rs.snapshot` — the full Arcium-wired program entry
- `Cargo.toml.snapshot` — Arcium + anchor-spl + blake3 dependency set
- `encrypted-ixs.lib.rs.snapshot` — both `payment_v3` and `beacon_bind` circuits

## What v1 (live, post-strip) keeps

- Public `register_beacon` instruction (one-shot per operator)
- Public `heartbeat` instruction
- `BeaconRegistry` account
- `BeaconRegistered`, `BeaconHeartbeat` events
- Registry-only errors: `InvalidRnsHash`, `InvalidRegionCode`, `OperatorMismatch`, `HeartbeatOverflow`
- Vanity program ID: `anon7uu8UtVoFgS8GCSfw2RqyphJhkN3xEjgPwznYDe`

## v2 revival plan (post-Frontier)

Single-program expansion: same program ID `anon7uu8…`, additive Anchor IDL.

Steps when v2 work resumes:

1. Re-add Arcium dependencies to `programs/anonbeta1/Cargo.toml`:
   - `anchor-spl = "0.32.1"` (only if SPL fee path returns)
   - `arcium-client = "=0.9.3"`
   - `arcium-macros = "=0.9.3"`
   - `arcium-anchor = "=0.9.3"`
   - `blake3 = "=1.8.2"` (if needed for commitments)
2. Switch `#[program]` back to `#[arcium_program]` in `lib.rs`
3. Restore the `beacon_bind` circuit in `encrypted-ixs/src/lib.rs` (still present in this archive)
4. Move the three instruction files back to `programs/anonbeta1/src/instructions/`
5. Re-add `PrivateBeaconRegistry` to `state.rs`
6. Re-add `BeaconBindCompleted` to `events.rs`
7. Re-add Arcium error variants to `errors.rs`
8. Re-add Arcium constants to `constants.rs`
9. Decide whether to keep the SOL/SPL registration fee or remove it
10. Add new Arcium-gated instructions for encrypted relay accounting:
    - Private relay counts
    - Private reward calculation
    - Encrypted operator stats
    - Optional: private sender/relay attribution

Existing v1 `BeaconRegistry` accounts remain valid; v2 capabilities are additive.

## Why not delete

This work is audited, compiles cleanly with Arcium 0.9.3, and represents real
MPC integration effort. Re-deriving it from scratch wastes time when v2 lands.
