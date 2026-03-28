import * as fs from 'fs';
import * as os from 'os';
import * as anchor from '@coral-xyz/anchor';
import {
  getCompDefAccOffset,
  getArciumProgramId,
  getArciumAccountBaseSeed,
  buildFinalizeCompDefTx,
} from '@arcium-hq/client';
import { PublicKey } from '@solana/web3.js';

const CIRCUIT_ID = 'payment_v3';

async function finalizeCompDef() {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.BleRevshare as anchor.Program;
  console.log('Program ID:', program.programId.toString());

  const walletPath = process.env.ANCHOR_WALLET || `${os.homedir()}/.config/solana/id.json`;
  const owner = anchor.web3.Keypair.fromSecretKey(
    new Uint8Array(JSON.parse(fs.readFileSync(walletPath, 'utf8')))
  );
  console.log('Signer:', owner.publicKey.toString());

  const offsetUint8Array = getCompDefAccOffset(CIRCUIT_ID);
  const compDefOffset = Buffer.from(offsetUint8Array).readUInt32LE(0);
  console.log('Comp def offset:', compDefOffset);

  const baseSeed = getArciumAccountBaseSeed('ComputationDefinitionAccount');
  const [compDefPDA] = PublicKey.findProgramAddressSync(
    [baseSeed, program.programId.toBuffer(), Buffer.from(offsetUint8Array)],
    getArciumProgramId()
  );
  console.log('Comp def PDA:', compDefPDA.toString());

  const accountInfo = await provider.connection.getAccountInfo(compDefPDA);
  if (!accountInfo) {
    throw new Error('Comp def account does not exist. Run init first.');
  }
  console.log('Current account size:', accountInfo.data.length, 'bytes');

  const tx = await buildFinalizeCompDefTx(provider, compDefOffset, program.programId);
  const { blockhash, lastValidBlockHeight } = await provider.connection.getLatestBlockhash('confirmed');
  tx.recentBlockhash = blockhash;
  tx.feePayer = owner.publicKey;

  const sim = await provider.connection.simulateTransaction(tx, [owner]);
  console.log('Simulation logs:', sim.value.logs);
  if (sim.value.err) {
    throw new Error(`Simulation failed: ${JSON.stringify(sim.value.err)}\n${sim.value.logs?.join('\n')}`);
  }

  tx.sign(owner);
  const sig = await provider.connection.sendRawTransaction(tx.serialize(), { skipPreflight: true });
  console.log('Sent tx:', sig);
  await provider.connection.confirmTransaction({ signature: sig, blockhash, lastValidBlockHeight }, 'confirmed');
  console.log('Finalized! Tx:', sig);

  const after = await provider.connection.getAccountInfo(compDefPDA);
  console.log('Account size after:', after?.data.length, 'bytes');
}

finalizeCompDef().catch((err) => {
  console.error('Error:', err);
  process.exit(1);
});
