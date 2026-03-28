import { before, describe, test } from "node:test";
import assert from "node:assert";
import { randomBytes } from "node:crypto";
import * as anchor from "@coral-xyz/anchor";
import { Keypair, PublicKey } from "@solana/web3.js";
import {
  getArciumEnv,
  getCompDefAccOffset,
  deserializeLE,
  getMXEAccAddress,
  getMempoolAccAddress,
  getCompDefAccAddress,
  getExecutingPoolAccAddress,
  getComputationAccAddress,
  getClusterAccAddress,
  getFeePoolAccAddress,
  getClockAccAddress,
} from "@arcium-hq/client";
import {
  createMint,
  createAccount,
  mintTo,
  getAccount,
} from "@solana/spl-token";
import { BleRevshare } from "../target/types/ble_revshare";

const TREASURY_WALLET = new PublicKey("DqyfDvr7yG4d3mtW6AiXgbuVM7GZWqn4RVARFbJxwtFc");
const MINT_DECIMALS = 6;
const MINT_SUPPLY = 1_000_000;
const PAYMENT_AMOUNT = new anchor.BN(1_000);

const PAYLOAD_TTL_SECS = 600; // 10 minutes

// 2% treasury cut; 30% of that to broadcaster, 70% to treasury
const TREASURY_CUT_BPS = 2;
const BROADCASTER_SHARE_PCT = 30;

const RECIPIENT_EXPECTED = PAYMENT_AMOUNT.muln(100 - TREASURY_CUT_BPS).divn(100).toNumber(); // 980
const BROADCASTER_EXPECTED = PAYMENT_AMOUNT.muln(TREASURY_CUT_BPS).divn(100).muln(BROADCASTER_SHARE_PCT).divn(100).toNumber(); // 6

anchor.setProvider(anchor.AnchorProvider.env());
const program = anchor.workspace.BleRevshare as anchor.Program<BleRevshare>;
const provider = anchor.getProvider() as anchor.AnchorProvider;
const payer = (provider.wallet as any).payer as Keypair;
const clusterOffset = getArciumEnv().arciumClusterOffset;

function arciumAccounts(computationOffset: anchor.BN) {
  return {
    computationAccount: getComputationAccAddress(clusterOffset, computationOffset),
    clusterAccount: getClusterAccAddress(clusterOffset),
    mxeAccount: getMXEAccAddress(program.programId),
    mempoolAccount: getMempoolAccAddress(clusterOffset),
    executingPool: getExecutingPoolAccAddress(clusterOffset),
    compDefAccount: getCompDefAccAddress(
      program.programId,
      Buffer.from(getCompDefAccOffset("payment_v3")).readUInt32LE()
    ),
    poolAccount: getFeePoolAccAddress(),
    clockAccount: getClockAccAddress(),
  };
}

function paymentReceipt(paymentId: Buffer): PublicKey {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("payment_receipt"), paymentId],
    program.programId
  )[0];
}

const ensureError = (err: unknown): Error => {
  if (err instanceof Error) return err;
  return new Error(`Non-Error thrown: ${JSON.stringify(err)}`);
};

describe("ble-revshare", () => {
  let mint: PublicKey;
  let senderTokenAccount: PublicKey;
  let recipientTokenAccount: PublicKey;
  let treasuryTokenAccount: PublicKey;
  let broadcasterTokenAccount: PublicKey;

  const recipient = Keypair.generate();
  const broadcaster = Keypair.generate();

  before(async () => {
    mint = await createMint(provider.connection, payer, payer.publicKey, null, MINT_DECIMALS);

    [senderTokenAccount, recipientTokenAccount, broadcasterTokenAccount] = await Promise.all([
      createAccount(provider.connection, payer, mint, payer.publicKey),
      createAccount(provider.connection, payer, mint, recipient.publicKey),
      createAccount(provider.connection, payer, mint, broadcaster.publicKey),
    ]);

    treasuryTokenAccount = await createAccount(
      provider.connection,
      payer,
      mint,
      TREASURY_WALLET,
      Keypair.generate() // off-curve owner requires explicit keypair
    );

    await mintTo(provider.connection, payer, mint, senderTokenAccount, payer, MINT_SUPPLY);
  });

  test("executes payment and distributes shares correctly (broadcaster present)", async () => {
    const computationOffset = new anchor.BN(randomBytes(8), "hex");
    const nonce = BigInt(deserializeLE(randomBytes(16)).toString());
    const paymentId = Buffer.from(randomBytes(32));
    const expiresAt = new anchor.BN(Math.floor(Date.now() / 1000) + PAYLOAD_TTL_SECS);

    await program.methods
      .executePayment(
        computationOffset,
        Array.from(paymentId),
        PAYMENT_AMOUNT,
        Array.from(randomBytes(32)),
        new anchor.BN(nonce.toString()),
        Array.from(payer.publicKey.toBytes()),
        expiresAt
      )
      .accountsPartial({
        payer: payer.publicKey,
        broadcaster: broadcaster.publicKey,
        recipient: recipient.publicKey,
        mint,
        paymentReceipt: paymentReceipt(paymentId),
        payerTokenAccount: senderTokenAccount,
        recipientTokenAccount,
        treasuryTokenAccount,
        broadcasterTokenAccount,
        ...arciumAccounts(computationOffset),
      })
      .signers([payer, broadcaster])
      .rpc();

    const [recipientBalance, broadcasterBalance] = await Promise.all([
      getAccount(provider.connection, recipientTokenAccount).then((a) => Number(a.amount)),
      getAccount(provider.connection, broadcasterTokenAccount).then((a) => Number(a.amount)),
    ]);

    assert.strictEqual(recipientBalance, RECIPIENT_EXPECTED);
    assert.strictEqual(broadcasterBalance, BROADCASTER_EXPECTED);
  });

  test("rejects duplicate payment_id (replay protection)", async () => {
    const computationOffset = new anchor.BN(randomBytes(8), "hex");
    const nonce = BigInt(deserializeLE(randomBytes(16)).toString());
    const paymentId = Buffer.from(randomBytes(32));
    const receipt = paymentReceipt(paymentId);

    const expiresAt = new anchor.BN(Math.floor(Date.now() / 1000) + PAYLOAD_TTL_SECS);

    const sendPayment = () =>
      program.methods
        .executePayment(
          computationOffset,
          Array.from(paymentId),
          PAYMENT_AMOUNT,
          Array.from(randomBytes(32)),
          new anchor.BN(nonce.toString()),
          Array.from(payer.publicKey.toBytes()),
          expiresAt
        )
        .accountsPartial({
          payer: payer.publicKey,
          broadcaster: broadcaster.publicKey,
          recipient: recipient.publicKey,
          mint,
          paymentReceipt: receipt,
          payerTokenAccount: senderTokenAccount,
          recipientTokenAccount,
          treasuryTokenAccount,
          broadcasterTokenAccount,
          ...arciumAccounts(computationOffset),
        })
        .signers([payer, broadcaster])
        .rpc();

    await sendPayment();
    await assert.rejects(sendPayment, /already in use/i);
  });

  test("rejects invalid treasury account", async () => {
    const computationOffset = new anchor.BN(randomBytes(8), "hex");
    const nonce = BigInt(deserializeLE(randomBytes(16)).toString());
    const paymentId = Buffer.from(randomBytes(32));
    const expiresAt = new anchor.BN(Math.floor(Date.now() / 1000) + PAYLOAD_TTL_SECS);

    // senderTokenAccount is owned by payer, not TREASURY_WALLET — valid for triggering InvalidTreasury
    try {
      await program.methods
        .executePayment(
          computationOffset,
          Array.from(paymentId),
          PAYMENT_AMOUNT,
          Array.from(randomBytes(32)),
          new anchor.BN(nonce.toString()),
          Array.from(payer.publicKey.toBytes()),
          expiresAt
        )
        .accountsPartial({
          payer: payer.publicKey,
          broadcaster: broadcaster.publicKey,
          recipient: recipient.publicKey,
          mint,
          paymentReceipt: paymentReceipt(paymentId),
          payerTokenAccount: senderTokenAccount,
          recipientTokenAccount,
          treasuryTokenAccount: senderTokenAccount,
          broadcasterTokenAccount,
          ...arciumAccounts(computationOffset),
        })
        .signers([payer, broadcaster])
        .rpc();

      assert.fail("Expected InvalidTreasury error");
    } catch (err) {
      const error = ensureError(err);
      assert.match(error.message, /InvalidTreasury/);
    }
  });

  test("executes payment without broadcaster (all treasury cut goes to treasury)", async () => {
    const computationOffset = new anchor.BN(randomBytes(8), "hex");
    const nonce = BigInt(deserializeLE(randomBytes(16)).toString());
    const paymentId = Buffer.from(randomBytes(32));
    const expiresAt = new anchor.BN(Math.floor(Date.now() / 1000) + PAYLOAD_TTL_SECS);

    const [recipientBefore, treasuryBefore] = await Promise.all([
      getAccount(provider.connection, recipientTokenAccount).then((a) => Number(a.amount)),
      getAccount(provider.connection, treasuryTokenAccount).then((a) => Number(a.amount)),
    ]);

    await program.methods
      .executePayment(
        computationOffset,
        Array.from(paymentId),
        PAYMENT_AMOUNT,
        Array.from(randomBytes(32)),
        new anchor.BN(nonce.toString()),
        Array.from(payer.publicKey.toBytes()),
        expiresAt
      )
      .accountsPartial({
        payer: payer.publicKey,
        broadcaster: null,
        recipient: recipient.publicKey,
        mint,
        paymentReceipt: paymentReceipt(paymentId),
        payerTokenAccount: senderTokenAccount,
        recipientTokenAccount,
        treasuryTokenAccount,
        broadcasterTokenAccount: null,
        ...arciumAccounts(computationOffset),
      })
      .signers([payer])
      .rpc();

    const TREASURY_NO_BROADCASTER = PAYMENT_AMOUNT.muln(TREASURY_CUT_BPS).divn(100).toNumber(); // 20

    const [recipientAfter, treasuryAfter] = await Promise.all([
      getAccount(provider.connection, recipientTokenAccount).then((a) => Number(a.amount)),
      getAccount(provider.connection, treasuryTokenAccount).then((a) => Number(a.amount)),
    ]);

    assert.strictEqual(recipientAfter - recipientBefore, RECIPIENT_EXPECTED);         // +980
    assert.strictEqual(treasuryAfter - treasuryBefore, TREASURY_NO_BROADCASTER);     // +20
  });

  test("rejects broadcaster token account provided without broadcaster co-signing", async () => {
    const computationOffset = new anchor.BN(randomBytes(8), "hex");
    const nonce = BigInt(deserializeLE(randomBytes(16)).toString());
    const paymentId = Buffer.from(randomBytes(32));
    const expiresAt = new anchor.BN(Math.floor(Date.now() / 1000) + PAYLOAD_TTL_SECS);

    try {
      await program.methods
        .executePayment(
          computationOffset,
          Array.from(paymentId),
          PAYMENT_AMOUNT,
          Array.from(randomBytes(32)),
          new anchor.BN(nonce.toString()),
          Array.from(payer.publicKey.toBytes()),
          expiresAt
        )
        .accountsPartial({
          payer: payer.publicKey,
          broadcaster: null,
          recipient: recipient.publicKey,
          mint,
          paymentReceipt: paymentReceipt(paymentId),
          payerTokenAccount: senderTokenAccount,
          recipientTokenAccount,
          treasuryTokenAccount,
          broadcasterTokenAccount, // token account supplied but no co-signer
          ...arciumAccounts(computationOffset),
        })
        .signers([payer])
        .rpc();

      assert.fail("Expected BroadcasterSignatureRequired error");
    } catch (err) {
      const error = ensureError(err);
      assert.match(error.message, /BroadcasterSignatureRequired/);
    }
  });

  test("rejects expired payment payload", async () => {
    const computationOffset = new anchor.BN(randomBytes(8), "hex");
    const nonce = BigInt(deserializeLE(randomBytes(16)).toString());
    const paymentId = Buffer.from(randomBytes(32));
    const expiredAt = new anchor.BN(Math.floor(Date.now() / 1000) - 60); // 60 seconds in the past — ensures simulation also fails

    try {
      await program.methods
        .executePayment(
          computationOffset,
          Array.from(paymentId),
          PAYMENT_AMOUNT,
          Array.from(randomBytes(32)),
          new anchor.BN(nonce.toString()),
          Array.from(payer.publicKey.toBytes()),
          expiredAt
        )
        .accountsPartial({
          payer: payer.publicKey,
          broadcaster: broadcaster.publicKey,
          recipient: recipient.publicKey,
          mint,
          paymentReceipt: paymentReceipt(paymentId),
          payerTokenAccount: senderTokenAccount,
          recipientTokenAccount,
          treasuryTokenAccount,
          broadcasterTokenAccount,
          ...arciumAccounts(computationOffset),
        })
        .signers([payer, broadcaster])
        .rpc();

      assert.fail("Expected PaymentExpired error");
    } catch (err) {
      const error = ensureError(err);
      assert.match(error.message, /PaymentExpired/);
    }
  });
});
