import * as fs from 'fs';
import * as os from 'os';
import * as anchor from '@coral-xyz/anchor';
import {
  getCompDefAccOffset,
  buildFinalizeCompDefTx,
  getMXEAccAddress,
  getArciumAccountBaseSeed,
  getArciumProgramId
} from '@arcium-hq/client';
import { PublicKey } from '@solana/web3.js';

// The ID used in the program for payment stats
const CIRCUIT_ID = 'payment_stats';
const USE_OFFCHAIN_CIRCUIT = true;

async function initCompDefFinal() {
  console.log('🚀 Starting FINAL Computation Definition initialization for payment_stats...');

  // Setup provider
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  // Get program
  const program = anchor.workspace.BleRevshare as anchor.Program;
  console.log('📋 Using program ID:', program.programId.toString());

  // Get owner keypair - using correct path from Anchor.toml
  const walletPath = process.env.ANCHOR_WALLET || `${os.homedir()}/.config/solana/id.json`;
  console.log('🔑 Using wallet:', walletPath);

  const owner = anchor.web3.Keypair.fromSecretKey(
    new Uint8Array(JSON.parse(fs.readFileSync(walletPath, 'utf8')))
  );
  console.log('👤 Owner public key:', owner.publicKey.toString());

  // Get computation definition PDA using correct method
  const baseSeedCompDefAcc = getArciumAccountBaseSeed("ComputationDefinitionAccount");
  const offsetUint8Array = getCompDefAccOffset(CIRCUIT_ID);
  const offsetBuffer = Buffer.from(offsetUint8Array);

  const [compDefPDA] = PublicKey.findProgramAddressSync(
    [baseSeedCompDefAcc, program.programId.toBuffer(), offsetBuffer],
    getArciumProgramId()
  );

  console.log('🎯 Computation definition PDA:', compDefPDA.toString());
  console.log('🔢 Offset:', Buffer.from(offsetUint8Array).readUInt32LE(0));

  try {
    // STEP 1: Initialize the computation definition account
    console.log('\n=== STEP 1: Initializing computation definition account ===');

    // Проверяем существует ли аккаунт
    const existingAccount = await provider.connection.getAccountInfo(compDefPDA);
    if (existingAccount) {
      console.log('✅ Computation definition account already exists');
    } else {
      console.log('📝 Creating new computation definition account...');
      try {
        const initSig = await program.methods
          .initPaymentStatsCompDef()
          .accounts({
            compDefAccount: compDefPDA,
            payer: owner.publicKey,
            mxeAccount: getMXEAccAddress(program.programId),
          })
          .signers([owner])
          .rpc({
            commitment: 'confirmed',
          });
        console.log('✅ Initialization transaction:', initSig);
        // Ждем подтверждения
        await new Promise(resolve => setTimeout(resolve, 2000));
      } catch (err: any) {
        console.error('❌ Failed to initialize computation definition:', err.message);
        throw err;
      }
    }

    // STEP 2: Circuit is loaded from IPFS automatically
    console.log('\n=== STEP 2: Computation Definition Setup ===');
    console.log('🌐 Circuit Source: OffChain (IPFS)');

    // STEP 3: Finalize the computation definition is SKIPPED as instructed
    console.log('\n=== STEP 3: Finalization skipped (OffChain circuit) ===');

    console.log('\n🎉 SUCCESS! Computation definition is ready!');

  } catch (error) {
    console.error('❌ Error during computation definition setup:', error);

    // Enhanced error logging
    if (error.logs) {
      console.error('📜 Transaction logs:');
      error.logs.forEach((log: string) => console.error('  ', log));
    }

    process.exit(1);
  }
}

// Run the script
initCompDefFinal().catch((error) => {
  console.error('💥 Fatal error:', error);
  process.exit(1);
});
