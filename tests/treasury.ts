import * as anchor from "@project-serum/anchor";
import * as web3 from "@solana/web3.js";
import { Keypair, SystemProgram, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import { Program } from "@project-serum/anchor";
import { Treasury } from "../target/types/treasury";
import { TOKEN_PROGRAM_ID, Token, MintLayout } from "@solana/spl-token";
import {
  createAssociatedTokenAccountInstruction,
  getAssociatedTokenAccountAddress,
} from "./helpers/tokenHelpers";

describe("treasury", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.Provider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Treasury as Program<Treasury>;
  let governor = null;
  let governorBump = null;
  let treasury = null;
  let treasuryBump = null;
  let transaction = web3.Keypair.generate();
  let ix = null;
  let mint = web3.Keypair.generate();
  let treasuryTokenAccount = null;
  let userTokenAccount = null;
  let mintAuthority = Keypair.generate();

  it("config", async () => {
    let [_governor, _governorBump] = await web3.PublicKey.findProgramAddress(
      [anchor.utils.bytes.utf8.encode("governor")],
      program.programId
    );
    governor = _governor;
    governorBump = _governorBump;

    let [_treasury, _treasuryBump] = await web3.PublicKey.findProgramAddress(
      [anchor.utils.bytes.utf8.encode("treasury")],
      program.programId
    );
    treasury = _treasury;
    treasuryBump = _treasuryBump;
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(
        treasury,
        5 * web3.LAMPORTS_PER_SOL
      ),
      "confirmed"
    );
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(
        mintAuthority.publicKey,
        5 * web3.LAMPORTS_PER_SOL
      ),
      "confirmed"
    );
    ix = web3.SystemProgram.transfer({
      fromPubkey: treasury,
      toPubkey: provider.wallet.publicKey,
      lamports: 0.000005 * web3.LAMPORTS_PER_SOL,
    });

    treasuryTokenAccount = await getAssociatedTokenAccountAddress(
      treasury,
      mint.publicKey
    );

    userTokenAccount = await getAssociatedTokenAccountAddress(
      provider.wallet.publicKey,
      mint.publicKey
    );
  });

  it("create governor", async () => {
    const tx = await program.rpc.createGovernor(governorBump, {
      accounts: {
        creator: provider.wallet.publicKey,
        governor: governor,
        systemProgram: web3.SystemProgram.programId,
      },
    });
    console.log("create gov sig", tx);
  });

  //simple transfer out of treasury
  it("create transaction", async () => {
    let pid = ix.programId;
    let accs = ix.keys;
    let data = ix.data;

    const tx = await program.rpc.createTransaction(pid, accs, data, {
      accounts: {
        creator: provider.wallet.publicKey,
        transaction: transaction.publicKey,
        governor: governor,
      },
      instructions: [
        await program.account.transaction.createInstruction(transaction, 1000),
      ],
      signers: [transaction],
    });

    let queuedTx = await program.account.transaction.fetch(
      transaction.publicKey
    );
  });
  //execute lamps transfer
  it("execute transaction", async () => {
    const tx = await program.rpc.executeTransaction(treasuryBump, {
      accounts: {
        transaction: transaction.publicKey,
        governor: governor,
        treasury: treasury,
      },
      remainingAccounts: ix.keys
        .map((meta: any) =>
          meta.pubkey.equals(treasury) ? { ...meta, isSigner: false } : meta
        )
        .concat({
          pubkey: web3.SystemProgram.programId,
          isWritable: false,
          isSigner: false,
        }),
    });
  });

  //gonna try a token account
  it("token config", async () => {
    let payer = provider.wallet.publicKey;

    const tx = await program.rpc.empty({
      accounts: {},
      instructions: [
        web3.SystemProgram.createAccount({
          fromPubkey: payer,
          newAccountPubkey: mint.publicKey,
          space: MintLayout.span,
          lamports: await provider.connection.getMinimumBalanceForRentExemption(
            MintLayout.span
          ),
          programId: TOKEN_PROGRAM_ID,
        }),
        //init the mint
        Token.createInitMintInstruction(
          TOKEN_PROGRAM_ID,
          mint.publicKey,
          0,
          mintAuthority.publicKey,
          mintAuthority.publicKey
        ),
        //create token account for new member card
        createAssociatedTokenAccountInstruction(
          mint.publicKey,
          treasuryTokenAccount,
          treasury,
          payer
        ),
        //create token account for user
        createAssociatedTokenAccountInstruction(
          mint.publicKey,
          userTokenAccount,
          payer,
          payer
        ),
      ],
      signers: [mint],
    });
  });

  it("create treasury token transfer ix", async () => {
    let transaction = web3.Keypair.generate();

    let NewToken = new Token(
      provider.connection,
      mint.publicKey,
      TOKEN_PROGRAM_ID,
      mintAuthority
    );
    await NewToken.mintTo(treasuryTokenAccount, mintAuthority, [], 100);

    let ix = Token.createTransferInstruction(
      TOKEN_PROGRAM_ID,
      treasuryTokenAccount,
      userTokenAccount,
      treasury,
      [],
      50
    );
    let pid = ix.programId;
    let accs = ix.keys;
    let data = ix.data;

    const tx = await program.rpc.createTransaction(pid, accs, data, {
      accounts: {
        creator: provider.wallet.publicKey,
        transaction: transaction.publicKey,
        governor: governor,
      },
      instructions: [
        await program.account.transaction.createInstruction(transaction, 1000),
      ],
      signers: [transaction],
    });

    let queuedTx = await program.account.transaction.fetch(
      transaction.publicKey
    );

    let trezBalance = await provider.connection.getTokenAccountBalance(
      treasuryTokenAccount
    );
    console.log("trez balance: ", trezBalance.value.uiAmount);
    let userBalance = await provider.connection.getTokenAccountBalance(
      userTokenAccount
    );
    console.log("user balance: ", userBalance.value.uiAmount);
    //let transferIx = Token.createTransferInstruction(TOKEN_PROGRAM_ID);

    const transferTx = await program.rpc.executeTransaction(treasuryBump, {
      accounts: {
        transaction: transaction.publicKey,
        governor: governor,
        treasury: treasury,
      },
      remainingAccounts: ix.keys
        .map((meta: any) =>
          meta.pubkey.equals(treasury) ? { ...meta, isSigner: false } : meta
        )
        .concat({
          pubkey: TOKEN_PROGRAM_ID,
          isWritable: false,
          isSigner: false,
        }),
    });

    //always append the programID, pass in accounts via remainingAccounts

    let trezBalance2 = await provider.connection.getTokenAccountBalance(
      treasuryTokenAccount
    );
    console.log("trez balance: ", trezBalance2.value.uiAmount);
    let userBalance2 = await provider.connection.getTokenAccountBalance(
      userTokenAccount
    );
    console.log("user balance: ", userBalance2.value.uiAmount);
  });
});
