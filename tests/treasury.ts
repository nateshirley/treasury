import * as anchor from "@project-serum/anchor";
import * as web3 from "@solana/web3.js";
import { Program } from "@project-serum/anchor";
import { Treasury } from "../target/types/treasury";

describe("treasury", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.Provider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Treasury as Program<Treasury>;
  let executor = null;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.rpc.initialize({});
    console.log("Your transaction signature", tx);
    let [_executor, bump] = await web3.PublicKey.findProgramAddress(
      [anchor.utils.bytes.utf8.encode("execute")],
      program.programId
    );
    executor = _executor;
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(
        _executor,
        5 * web3.LAMPORTS_PER_SOL
      ),
      "confirmed"
    );
  });

  it("pass ix", async () => {
    let receiver = web3.Keypair.generate();
    console.log(executor.toBase58());

    let ix = web3.SystemProgram.transfer({
      fromPubkey: executor,
      toPubkey: provider.wallet.publicKey,
      lamports: 0.000005 * web3.LAMPORTS_PER_SOL,
    });
    console.log(ix);

    let vec = [1, 2, 3, 4, 5];

    await program.rpc.executeInstruction(ix.programId, ix.keys, ix.data, {
      accounts: {
        from: executor,
        to: provider.wallet.publicKey,
        systemProgram: web3.SystemProgram.programId,
      },
    });
  });
});
