import * as fs from 'fs';
import * as os from 'os';
import * as anchor from '@coral-xyz/anchor';
import BN from 'bn.js';
import {
  getCompDefAccOffset,
  getMXEAccAddress,
  getArciumAccountBaseSeed,
  getArciumProgramId,
  getLookupTableAddress,
  getArciumProgram,
} from '@arcium-hq/client';
import { PublicKey, AddressLookupTableProgram } from '@solana/web3.js';

// The ID used in the program for payment stats
const CIRCUIT_ID = 'payment_v3';

async function initCompDefFinal() {
  console.log('🚀 Starting FINAL Computation Definition initialization for payment_stats...');

  // Setup provider
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  // Get program
  const program = anchor.workspace.BleRevshare as anchor.Program;
  console.log('📋 Using program ID:', program.programId.toString());

  // Get owner keypair
  const walletPath = process.env.ANCHOR_WALLET || `${os.homedir()}/.config/solana/id.json`;
  console.log('🔑 Using wallet:', walletPath);

  const owner = anchor.web3.Keypair.fromSecretKey(
    new Uint8Array(JSON.parse(fs.readFileSync(walletPath, 'utf8')))
  );
  console.log('👤 Owner public key:', owner.publicKey.toString());

  // Get computation definition PDA
  const baseSeedCompDefAcc = getArciumAccountBaseSeed("ComputationDefinitionAccount");
  const offsetUint8Array = getCompDefAccOffset(CIRCUIT_ID);
  const offsetBuffer = Buffer.from(offsetUint8Array);

  const [compDefPDA] = PublicKey.findProgramAddressSync(
    [baseSeedCompDefAcc, program.programId.toBuffer(), offsetBuffer],
    getArciumProgramId()
  );

  console.log('🎯 Computation definition PDA:', compDefPDA.toString());
  console.log('🔢 Offset:', Buffer.from(offsetUint8Array).readUInt32LE(0));

  const mxeAddress = getMXEAccAddress(program.programId);

  try {
    // STEP 1: Initialize the computation definition account
    console.log('\n=== STEP 1: Initializing computation definition account ===');

    // Fetch lutOffsetSlot from MXE account for LUT derivation
    let lutOffsetSlot = new BN(0);
    try {
      const arciumProg = getArciumProgram(provider);
      const mxeData = await arciumProg.account['mxeAccount'].fetch(mxeAddress);
      lutOffsetSlot = (mxeData as any).lutOffsetSlot;
      console.log('🔢 LUT offset slot:', lutOffsetSlot.toString());
    } catch (e) {
      console.warn('⚠️  Could not fetch lutOffsetSlot, using 0:', e);
    }

    const addressLookupTable = getLookupTableAddress(program.programId, lutOffsetSlot);
    console.log('🗂️  Address Lookup Table:', addressLookupTable.toString());

    const existingAccount = await provider.connection.getAccountInfo(compDefPDA);
    if (existingAccount) {
      console.log('⚠️  Comp def account exists — closing it for re-initialization...');
      // Drain lamports to payer by sending a transfer (relies on payer being the owner)
      // Anchor init will fail if account exists, so we need to close it first.
      // Use Arcium's close if available, otherwise inform the user.
      console.log('ℹ️  Close the account manually at:', compDefPDA.toString());
      console.log('ℹ️  Or run: solana account', compDefPDA.toString(), '--output json');
      console.log('ℹ️  Then re-run this script.');
      console.log('\nAttempting init anyway (works if Arcium program supports re-init)...');
    }

    try {
      const tx = await program.methods
        .initPaymentStatsCompDef()
        .accounts({
          compDefAccount: compDefPDA,
          payer: owner.publicKey,
          mxeAccount: mxeAddress,
          addressLookupTable,
          lutProgram: AddressLookupTableProgram.programId,
        })
        .signers([owner])
        .transaction();

      // Simulate first to catch on-chain errors with logs
      const { blockhash, lastValidBlockHeight } = await provider.connection.getLatestBlockhash('confirmed');
      tx.recentBlockhash = blockhash;
      tx.feePayer = owner.publicKey;
      tx.sign(owner);

      const sim = await provider.connection.simulateTransaction(tx);
      console.log('🔍 Simulation logs:', sim.value.logs);
      if (sim.value.err) {
        throw new Error(`Simulation failed: ${JSON.stringify(sim.value.err)}\nLogs:\n${sim.value.logs?.join('\n')}`);
      }

      const rawTx = tx.serialize();
      const sig = await provider.connection.sendRawTransaction(rawTx, { skipPreflight: true });
      console.log('📤 Sent tx:', sig);
      await provider.connection.confirmTransaction({ signature: sig, blockhash, lastValidBlockHeight }, 'confirmed');
      console.log('✅ Initialization transaction:', sig);
      await new Promise(resolve => setTimeout(resolve, 2000));
    } catch (err: any) {
      if (err.message?.includes('already in use') || err.message?.includes('already initialized')) {
        console.error('❌ Comp def account already in use.');
      } else {
        console.error('❌ Failed:', err.message);
      }
      throw err;
    }

    console.log('\n=== STEP 2: Computation Definition Setup ===');
    console.log('🌐 Circuit Source: OffChain (IPFS)');
    console.log('\n=== STEP 3: Finalization skipped (OffChain circuit) ===');
    console.log('\n🎉 SUCCESS! Computation definition is ready!');

  } catch (error) {
    console.error('❌ Error during computation definition setup:', error);

    if (error.logs) {
      console.error('📜 Transaction logs:');
      error.logs.forEach((log: string) => console.error('  ', log));
    }

    process.exit(1);
  }
}

initCompDefFinal().catch((error) => {
  console.error('💥 Fatal error:', error);
  process.exit(1);
});
