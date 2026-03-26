import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Keypair, PublicKey, AddressLookupTableProgram } from "@solana/web3.js";
import { BleRevshare } from "../target/types/ble_revshare";
import { randomBytes } from "crypto";
import {
  awaitComputationFinalization,
  getArciumEnv,
  getCompDefAccOffset,
  getArciumAccountBaseSeed,
  getArciumProgramId,
  buildFinalizeCompDefTx,
  deserializeLE,
  getMXEAccAddress,
  getMempoolAccAddress,
  getCompDefAccAddress,
  getExecutingPoolAccAddress,
  getComputationAccAddress,
  getClusterAccAddress,
  getFeePoolAccAddress,
  getClockAccAddress,
  getLookupTableAddress,
  getArciumProgram,
} from "@arcium-hq/client";
import {
  createMint,
  createAccount,
  mintTo,
  getAccount,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import * as fs from "fs";
import * as os from "os";
import { expect } from "chai";

const CLUSTER_OFFSET: number | null = null;

function getClusterAccount(): PublicKey {
  if (CLUSTER_OFFSET !== null) {
    return getClusterAccAddress(CLUSTER_OFFSET);
  } else {
    return getClusterAccAddress(getArciumEnv().arciumClusterOffset);
  }
}

describe("ble-revshare", () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const program = anchor.workspace.BleRevshare as Program<BleRevshare>;
  const provider = anchor.getProvider() as anchor.AnchorProvider;
  const payer = (provider.wallet as any).payer as Keypair;

  const clusterAccount = getClusterAccount();

  let mint: PublicKey;
  let senderTokenAccount: PublicKey;
  let recipientTokenAccount: PublicKey;
  let treasuryTokenAccount: PublicKey;
  let broadcasterTokenAccount: PublicKey;

  const recipient = Keypair.generate();
  const treasury = Keypair.generate();
  const broadcaster = Keypair.generate();

  before(async () => {
    // Setup tokens
    mint = await createMint(
      provider.connection,
      payer,
      payer.publicKey,
      null,
      6
    );

    senderTokenAccount = await createAccount(
      provider.connection,
      payer,
      mint,
      payer.publicKey
    );

    recipientTokenAccount = await createAccount(
      provider.connection,
      payer,
      mint,
      recipient.publicKey
    );

    treasuryTokenAccount = await createAccount(
      provider.connection,
      payer,
      mint,
      treasury.publicKey
    );

    broadcasterTokenAccount = await createAccount(
      provider.connection,
      payer,
      mint,
      broadcaster.publicKey
    );

    await mintTo(
      provider.connection,
      payer,
      mint,
      senderTokenAccount,
      payer,
      1000000 // 1 USDC
    );
  });

  // it("Initializes comp def for payment_stats", async () => {
  //     const sig = await initCompDef(
  //         program,
  //         payer,
  //         "payment_stats",
  //         "initPaymentStatsCompDef"
  //     );
  //     console.log("payment_stats initialized", sig);
  // });

  it("Adds token to whitelist", async () => {
    await program.methods
      .initWhitelistToken()
      .accounts({
        admin: payer.publicKey,
        mint: mint,
      })
      .rpc();

    const [whitelistEntry] = PublicKey.findProgramAddressSync(
      [Buffer.from("whitelist"), mint.toBuffer()],
      program.programId
    );

    const account = await program.account.whitelistEntry.fetch(whitelistEntry);
    expect(account.mint.toString()).to.equal(mint.toString());
  });

  it("Executes payment with Broadcaster (70/30)", async () => {
    const computationOffset = new anchor.BN(randomBytes(8), "hex");
    const nonce = BigInt(deserializeLE(randomBytes(16)).toString());
    const amount = new anchor.BN(1000); // 1000 units

    // We expect 300 to broadcaster, 700 to recipient

    // Signer PDA
    const [signPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("ArciumSignerAccount")],
      program.programId
    );

    try {
      await program.methods
        .executePayment(
          computationOffset,
          amount,
          new anchor.BN(nonce.toString()),
          [...payer.publicKey.toBytes()]
        )
        .accountsPartial({
          payer: payer.publicKey,
          broadcaster: broadcaster.publicKey,
          recipient: recipient.publicKey,
          mint: mint,
          payerTokenAccount: senderTokenAccount,
          recipientTokenAccount: recipientTokenAccount,
          treasuryTokenAccount: treasuryTokenAccount,
          broadcasterTokenAccount: broadcasterTokenAccount,
          // Arcium accounts
          computationAccount: getComputationAccAddress(
            getArciumEnv().arciumClusterOffset,
            computationOffset
          ),
          clusterAccount,
          mxeAccount: getMXEAccAddress(program.programId),
          mempoolAccount: getMempoolAccAddress(
            getArciumEnv().arciumClusterOffset
          ),
          executingPool: getExecutingPoolAccAddress(
            getArciumEnv().arciumClusterOffset
          ),
          compDefAccount: getCompDefAccAddress(
            program.programId,
            Buffer.from(getCompDefAccOffset("payment_stats")).readUInt32LE()
          ),
          poolAccount: getFeePoolAccAddress(),
          clockAccount: getClockAccAddress(),
          // Actually usually ARCIUM_FEE_POOL_ACCOUNT_ADDRESS is hardcoded or imported.
          // Client usually doesn't need to pass it if it's constant unless IDL requires it.
          // But my IDL does require it.
          // I'll grab it from checking IDL or just rely on anchor to fill if seeds match, but here address is constant.
          // Wait, in lib.rs I used `ARCIUM_FEE_POOL_ACCOUNT_ADDRESS`.
          // I should import it from client if available or defined.
          // Checking escrow.ts, it was passed as valid account? No, `readKpJson`...
          // escrow.ts did NOT import it.
          // Wait, `initializeEscrow` in escrow.ts didn't pass `poolAccount` manually?
          // Ah, looking at `escrow.ts` lines 115-130: NO poolAccount passed!
          // Anchor can resolve if default? No.
          // Wait, Anchor `accountsPartial` might not enforce all strictly if they are inferred.
          // But `poolAccount` has `address = ...` constraint.
          // The constraint is on the PROGRAM side.
          // The client must pass the account with that address.
          // If the IDL has a default address, Anchor uses it.
          // BUT `ARCIUM_FEE_POOL_ACCOUNT_ADDRESS` is a constant in the program.
          // How does Client know it?
          // Usually we need to pass it.
          // In escrow.ts, I see `mxeAccount`, `mempoolAccount` etc. but no `poolAccount`.
          // Let me re-read `escrow.ts` InitializeEscrow accounts carefully.
          // lines 115-130: OWNER, ComputationAccount, ClusterAccount, MXEAccount, Mempool, ExecutingPool, CompDef, Escrow.
          // THAT'S IT.
          // But `lib.rs` InitializeEscrow struct has `pool_account` and `clock_account`.
          // Why aren't they passed?
          // Maybe they are *optional* or inferred by Anchor if standard?
          // Or maybe `accountsPartial` lets me skip them and it fails?
          // OR the IDL has them marked such that Anchor resolves them?
          // Arcium Anchor client might inject them?
          // I will assume I need to pass them or let Arcium client handle.
          // I'll check `arcium-hq/client` exports.
        })
        .signers([payer, broadcaster])
        .rpc();

      // Check balances
      const recipientBalance = (
        await getAccount(provider.connection, recipientTokenAccount)
      ).amount;
      const broadcasterBalance = (
        await getAccount(provider.connection, broadcasterTokenAccount)
      ).amount;

      expect(recipientBalance.toString()).to.equal("700");
      expect(broadcasterBalance.toString()).to.equal("300");
    } catch (e) {
      // If poolAccount is missing try explicit pass
      console.error(e);
      throw e;
    }
  });

  async function initCompDef(
    program: Program<BleRevshare>,
    owner: anchor.web3.Keypair,
    compDefName: string,
    methodName: "initPaymentStatsCompDef"
  ): Promise<string> {
    const baseSeedCompDefAcc = getArciumAccountBaseSeed(
      "ComputationDefinitionAccount"
    );
    const offset = getCompDefAccOffset(compDefName);

    const compDefPDA = PublicKey.findProgramAddressSync(
      [baseSeedCompDefAcc, program.programId.toBuffer(), offset],
      getArciumProgramId()
    )[0];

    const mxeAddress = getMXEAccAddress(program.programId);

    let lutOffsetSlot = new anchor.BN(0);
    try {
      const arciumProg = getArciumProgram(provider);
      const mxeData = await arciumProg.account["mxeAccount"].fetch(mxeAddress);
      lutOffsetSlot = (mxeData as any).lutOffsetSlot;
    } catch (e) {
      console.warn("Could not fetch lutOffsetSlot, using 0:", e);
    }

    const addressLookupTable = getLookupTableAddress(
      program.programId,
      lutOffsetSlot
    );

    const sig = await program.methods[methodName]()
      .accounts({
        compDefAccount: compDefPDA,
        payer: owner.publicKey,
        mxeAccount: mxeAddress,
        addressLookupTable,
        lutProgram: AddressLookupTableProgram.programId,
      } as any)
      .signers([owner])
      .rpc({
        commitment: "confirmed",
      });

    const finalizeTx = await buildFinalizeCompDefTx(
      provider,
      Buffer.from(offset).readUInt32LE(),
      program.programId
    );

    const latestBlockhash = await provider.connection.getLatestBlockhash();
    finalizeTx.recentBlockhash = latestBlockhash.blockhash;
    finalizeTx.lastValidBlockHeight = latestBlockhash.lastValidBlockHeight;

    finalizeTx.sign(owner);

    await provider.sendAndConfirm(finalizeTx);

    return sig;
  }
});
