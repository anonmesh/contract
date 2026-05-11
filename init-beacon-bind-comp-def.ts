import * as fs from 'fs';
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

const CIRCUIT_ID = 'beacon_bind';

async function initBeaconBindCompDef() {
  console.log('Starting beacon_bind computation definition init...');

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Anonbeta1 as anchor.Program;
  console.log('Program ID:', program.programId.toString());

  const walletPath = process.env.ANCHOR_WALLET || './anontestpair_01.json';
  const owner = anchor.web3.Keypair.fromSecretKey(
    new Uint8Array(JSON.parse(fs.readFileSync(walletPath, 'utf8')))
  );
  console.log('Owner:', owner.publicKey.toString());

  const baseSeedCompDefAcc = getArciumAccountBaseSeed("ComputationDefinitionAccount");
  const offsetUint8Array = getCompDefAccOffset(CIRCUIT_ID);
  const offsetBuffer = Buffer.from(offsetUint8Array);

  const [compDefPDA] = PublicKey.findProgramAddressSync(
    [baseSeedCompDefAcc, program.programId.toBuffer(), offsetBuffer],
    getArciumProgramId()
  );

  console.log('Comp def PDA:', compDefPDA.toString());
  console.log('Offset:', Buffer.from(offsetUint8Array).readUInt32LE(0));

  const mxeAddress = getMXEAccAddress(program.programId);
  console.log('MXE address:', mxeAddress.toString());

  let lutOffsetSlot = new BN(0);
  try {
    const arciumProg = getArciumProgram(provider);
    const mxeData = await arciumProg.account['mxeAccount'].fetch(mxeAddress);
    lutOffsetSlot = (mxeData as any).lutOffsetSlot;
    console.log('LUT offset slot:', lutOffsetSlot.toString());
  } catch (e) {
    console.warn('Could not fetch lutOffsetSlot, using 0:', e);
  }

  const addressLookupTable = getLookupTableAddress(program.programId, lutOffsetSlot);
  console.log('LUT address:', addressLookupTable.toString());

  const existingAccount = await provider.connection.getAccountInfo(compDefPDA);
  if (existingAccount) {
    console.log('Comp def account already exists at:', compDefPDA.toString());
    console.log('Skipping init (already done).');
    return;
  }

  try {
    const tx = await program.methods
      .initBeaconBindCompDef()
      .accounts({
        compDefAccount: compDefPDA,
        payer: owner.publicKey,
        mxeAccount: mxeAddress,
        addressLookupTable,
        lutProgram: AddressLookupTableProgram.programId,
      })
      .signers([owner])
      .transaction();

    const { blockhash, lastValidBlockHeight } = await provider.connection.getLatestBlockhash('confirmed');
    tx.recentBlockhash = blockhash;
    tx.feePayer = owner.publicKey;
    tx.sign(owner);

    const sim = await provider.connection.simulateTransaction(tx);
    console.log('Simulation logs:', sim.value.logs);
    if (sim.value.err) {
      throw new Error(`Simulation failed: ${JSON.stringify(sim.value.err)}\nLogs:\n${sim.value.logs?.join('\n')}`);
    }

    const rawTx = tx.serialize();
    const sig = await provider.connection.sendRawTransaction(rawTx, { skipPreflight: true });
    console.log('Sent tx:', sig);
    await provider.connection.confirmTransaction({ signature: sig, blockhash, lastValidBlockHeight }, 'confirmed');
    console.log('Init comp def tx confirmed:', sig);
  } catch (err: any) {
    console.error('Failed:', err.message);
    if (err.logs) {
      console.error('Logs:');
      err.logs.forEach((log: string) => console.error(' ', log));
    }
    process.exit(1);
  }

  console.log('beacon_bind computation definition initialized.');
}

initBeaconBindCompDef().catch((error) => {
  console.error('Fatal:', error);
  process.exit(1);
});
