use anchor_lang::prelude::*;
use arcium_anchor::prelude::*;
use arcium_client::idl::arcium::types::CallbackAccount;
use solana_keccak_hasher as keccak;

use super::beacon_bind_callback::BeaconBindCallback;
use crate::constants::COMP_DEF_OFFSET_BEACON_BIND;
use crate::errors::ErrorCode;
use crate::events::BeaconRegistered;
use crate::state::{BeaconRegistry, PrivateBeaconBinding};
use crate::{ArciumSignerAccount, ID, ID_CONST};

#[queue_computation_accounts("beacon_bind", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64)]
pub struct RegisterBeaconPrivate<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = 8 + BeaconRegistry::INIT_SPACE,
        seeds = [b"beacon", payer.key().as_ref()],
        bump,
    )]
    pub beacon: Account<'info, BeaconRegistry>,

    #[account(
        init,
        payer = payer,
        space = 8 + PrivateBeaconBinding::INIT_SPACE,
        seeds = [b"private_beacon", payer.key().as_ref()],
        bump,
    )]
    pub private_binding: Account<'info, PrivateBeaconBinding>,

    #[account(
        init_if_needed,
        space = 9,
        payer = payer,
        seeds = [b"ArciumSignerAccount"],
        bump,
    )]
    pub sign_pda_account: Account<'info, ArciumSignerAccount>,

    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Account<'info, MXEAccount>,

    #[account(
        mut,
        address = derive_mempool_pda!(mxe_account, ErrorCode::ClusterNotSet)
    )]
    /// CHECK: mempool_account
    pub mempool_account: UncheckedAccount<'info>,

    #[account(
        mut,
        address = derive_execpool_pda!(mxe_account, ErrorCode::ClusterNotSet)
    )]
    /// CHECK: executing_pool
    pub executing_pool: UncheckedAccount<'info>,

    #[account(
        mut,
        address = derive_comp_pda!(computation_offset, mxe_account, ErrorCode::ClusterNotSet)
    )]
    /// CHECK: computation_account
    pub computation_account: UncheckedAccount<'info>,

    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_BEACON_BIND))]
    pub comp_def_account: Box<Account<'info, ComputationDefinitionAccount>>,

    #[account(
        mut,
        address = derive_cluster_pda!(mxe_account, ErrorCode::ClusterNotSet)
    )]
    pub cluster_account: Box<Account<'info, Cluster>>,

    #[account(mut, address = ARCIUM_FEE_POOL_ACCOUNT_ADDRESS)]
    pub pool_account: Box<Account<'info, FeePool>>,

    #[account(mut, address = ARCIUM_CLOCK_ACCOUNT_ADDRESS)]
    pub clock_account: Box<Account<'info, ClockAccount>>,

    pub system_program: Program<'info, System>,
    pub arcium_program: Program<'info, Arcium>,
}

pub(crate) fn handler(
    ctx: Context<RegisterBeaconPrivate>,
    computation_offset: u64,
    encrypted_rns_dest_hash: [u8; 32],
    encrypted_region_code: [u8; 32],
    nonce: u128,
    pub_key: [u8; 32],
    region_code: [u8; 4],
    capabilities_bitmap: u32,
) -> Result<()> {
    // Derive the public binding id on-chain instead of accepting client-chosen
    // bytes. This keeps the encrypted RNS destination hash from being leaked by
    // an accidental or malicious binding_id = rns_dest_hash convention.
    let operator = ctx.accounts.payer.key();
    let binding_id: [u8; 32] = keccak::hashv(&[operator.as_ref(), &nonce.to_le_bytes()]).to_bytes();
    // One binding per operator is enforced by the operator-scoped init PDAs.
    require!(binding_id != [0u8; 32], ErrorCode::InvalidBindingId);
    require!(pub_key != [0u8; 32], ErrorCode::InvalidX25519Pubkey);
    require!(
        encrypted_rns_dest_hash != [0u8; 32],
        ErrorCode::InvalidCiphertext
    );
    require!(
        encrypted_region_code != [0u8; 32],
        ErrorCode::InvalidCiphertext
    );
    require!(
        region_code.iter().all(|b| (0x20..=0x7E).contains(b)),
        ErrorCode::InvalidRegionCode
    );

    let args = ArgBuilder::new()
        .x25519_pubkey(pub_key)
        .plaintext_u128(nonce)
        .encrypted_u128(encrypted_rns_dest_hash)
        .encrypted_u32(encrypted_region_code)
        .build();

    let now = Clock::get()?.unix_timestamp;
    let beacon_pda = ctx.accounts.beacon.key();
    let private_binding_pda = ctx.accounts.private_binding.key();

    {
        let beacon = &mut ctx.accounts.beacon;
        beacon.bump = ctx.bumps.beacon;
        beacon.operator = operator;
        beacon.private_binding = private_binding_pda;
        beacon.binding_id = binding_id;
        beacon.region_code = region_code;
        beacon.capabilities_bitmap = capabilities_bitmap;
        beacon.registered_at = now;
        beacon.binding_verified = false;
        beacon.binding_verified_at = 0;
        beacon.last_relay_at = 0;
        beacon.settlement_count = 0;
    }

    {
        let private_binding = &mut ctx.accounts.private_binding;
        private_binding.bump = ctx.bumps.private_binding;
        private_binding.operator = operator;
        private_binding.beacon_pda = beacon_pda;
        private_binding.binding_id = binding_id;
        private_binding.encrypted_commitment = [0u8; 32];
        private_binding.commitment_nonce = 0;
        private_binding.x25519_pubkey = pub_key;
        private_binding.verified = false;
        private_binding.queued_at = now;
        private_binding.verified_at = 0;
    }

    ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;

    let binding_acc = CallbackAccount {
        pubkey: private_binding_pda,
        is_writable: true,
    };
    let beacon_acc = CallbackAccount {
        pubkey: beacon_pda,
        is_writable: true,
    };

    // NOTE: emitted when the Arcium computation is queued. Indexers should wait
    // for BeaconBindCompleted before treating the binding as verified.
    emit!(BeaconRegistered {
        operator,
        beacon_pda,
        private_binding: private_binding_pda,
        binding_id,
        region_code,
        capabilities_bitmap,
        registered_at: now,
    });

    queue_computation(
        ctx.accounts,
        computation_offset,
        args,
        vec![BeaconBindCallback::callback_ix(
            computation_offset,
            &ctx.accounts.mxe_account,
            &[binding_acc, beacon_acc],
        )?],
        1,
        0,
    )?;

    Ok(())
}
