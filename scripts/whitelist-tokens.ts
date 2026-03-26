// scripts/whitelist-tokens.ts
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { BleRevshare } from "../target/types/ble_revshare";
import { PublicKey, SystemProgram } from "@solana/web3.js";

const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);

const program = anchor.workspace.BleRevshare as Program<BleRevshare>;

const SOL_MINT = new PublicKey("So11111111111111111111111111111111111111112"); // Wrapped SOL
const USDC_MINT = new PublicKey("4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU"); // Devnet USDC (replace if using different mint)

async function whitelistToken(mint: PublicKey) {
  const [whitelistEntry] = PublicKey.findProgramAddressSync(
    [Buffer.from("whitelist"), mint.toBuffer()],
    program.programId
  );

  console.log(`Whitelisting mint: ${mint.toBase58()}`);
  console.log(`PDA: ${whitelistEntry.toBase58()}`);

  const tx = await program.methods
    .initWhitelistToken()
    .accounts({
      admin: provider.wallet.publicKey,
      mint: mint,
      // ← Remove whitelistEntry entirely (Anchor resolves the PDA automatically)
    })
    .rpc({ commitment: "confirmed" });

  console.log(`✅ Successfully whitelisted ${mint.toBase58()} → Tx: ${tx}`);
}

async function main() {
  console.log("=== Whitelisting SOL and USDC on devnet ===\n");

  await whitelistToken(SOL_MINT);
  await whitelistToken(USDC_MINT);

  console.log("\n🎉 Whitelisting completed successfully!");
}

main().catch((err) => {
  console.error("❌ Error:", err);
  process.exit(1);
});
