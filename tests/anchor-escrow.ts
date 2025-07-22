import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AnchorEscrow } from "../target/types/anchor_escrow";
import { Keypair, LAMPORTS_PER_SOL, PUBLIC_KEY_LENGTH, PublicKey, TokenAmount } from "@solana/web3.js";
import { Account, Account as TokenAccount, createMint, getAccount, getOrCreateAssociatedTokenAccount, mintTo, Mint, getAssociatedTokenAddress, TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID } from "@solana/spl-token";


describe("anchor-escrow", () => {
  // Configure the client to use the local cluster.

  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.AnchorProvider.env();
  const program = anchor.workspace.anchorEscrow as Program<AnchorEscrow>;
  console.log("program id: ", program.programId);
  const maker = Keypair.generate();
  const taker = Keypair.generate();
  let mint_a: anchor.web3.PublicKey;
  let mint_b: anchor.web3.PublicKey;
  let maker_ata_a: Account;
  let taker_ata_b: Account;
  let maker_ata_b:anchor.web3.PublicKey;
  let taker_ata_a:anchor.web3.PublicKey;
  let seed = new anchor.BN(1);

  let escrow = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("escrow"), maker.publicKey.toBytes(), Buffer.from(seed.toArray("le", 8))], program.programId)[0];
  let vault: anchor.web3.PublicKey;


  before(async () => {
    //sending solana to maker and taker
    let sendSol = async (to: PublicKey, amount: number) => {
      let tx = new anchor.web3.Transaction().add(
        anchor.web3.SystemProgram.transfer({
          fromPubkey: anchor.AnchorProvider.env().publicKey,
          toPubkey: to,
          lamports: amount,
        })
      )
      let sig = await provider.sendAndConfirm(tx, [provider.wallet.payer]);
      console.log("sol sent succefully:,", sig);
    }
    await sendSol(maker.publicKey, 5 * LAMPORTS_PER_SOL);
    await sendSol(taker.publicKey, 5 * LAMPORTS_PER_SOL);

    let createMintAndAta = async (user: Keypair, amount: number): Promise<({ mint: PublicKey, ata: Account })> => {
      let mint = await createMint(provider.connection, provider.wallet.payer, provider.wallet.publicKey, null, 6);
      let ata = (await getOrCreateAssociatedTokenAccount(provider.connection, provider.wallet.payer, mint, user.publicKey));
      let sig = await mintTo(provider.connection, provider.wallet.payer, mint, ata.address, provider.wallet.payer, amount);
      let ata_info = await getAccount(provider.connection, ata.address);
      console.log(` maker or taker ${user.publicKey} : `, Number(ata_info.amount));
      return { mint: mint, ata: ata };
    }

    //now creating mint and minting this token to maker 
    ({ mint: mint_a, ata: maker_ata_a } = await createMintAndAta(maker, 1_0_000_000));
    ({ mint: mint_b, ata: taker_ata_b } = await createMintAndAta(taker, 2_0_000_000));
    maker_ata_b = await getAssociatedTokenAddress(mint_b,maker.publicKey ,true);
    taker_ata_a = await getAssociatedTokenAddress(mint_a , taker.publicKey,true);
    vault = await getAssociatedTokenAddress(mint_a, escrow, true);
    console.log("escorw: ", escrow.toBase58());
    console.log("vault: ", vault.toBase58());

  });

  it("making offer", async () => {
    let received_amount = new anchor.BN(Number(1_0_000_000));
    let amount = new anchor.BN(Number(5_000_000));

    const tx = await program.methods
      .initializeEscrow(seed, received_amount, amount)
      .accountsPartial({
        maker: maker.publicKey,
        mintA: mint_a,
        mintB: mint_b,
        makerAtaA: maker_ata_a.address,
        escrow: escrow,
        vault: vault,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      })
      .signers([maker])
      .rpc()
    let vaultInfo = await getAccount(provider.connection, vault);
    let makerAtaAInfo = await getAccount(provider.connection, maker_ata_a.address);
    console.log("deposited to escrow:✅ ", tx);
    console.log("after initialization amount of token vault a having: ",vaultInfo.amount);
    console.log("after initialization amount of token makerAta a having: ",makerAtaAInfo.amount);
  })


  it.skip("taking refund", async () => {

    const tx = await program.methods
      .takingRefund()
      .accountsPartial({
        maker: maker.publicKey,
        mintA: mint_a,
        makerAtaA: maker_ata_a.address,
        escrow: escrow,
        vault: vault,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      })
      .signers([maker])
      .rpc()
    let makerAtaAInfo = await getAccount(provider.connection, maker_ata_a.address);
    console.log("refunded succefully✅ ", tx);
    console.log("after refund amount of token makerAta a having: ",makerAtaAInfo.amount);
  })


    it("finalize the deal", async () => {
      const tx = await program.methods
        .finalizeDeal()
        .accountsPartial({
          taker:taker.publicKey,
          maker: maker.publicKey,
          mintA: mint_a,
          mintB:mint_b,
          takerAtaA:taker_ata_a,
          takerAtaB:taker_ata_b.address,
          makerAtaB:maker_ata_b,
          escrow: escrow,
          vault: vault,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        })
        .signers([taker])
        .rpc()
        let makerAtaBInfo = await getAccount(provider.connection, maker_ata_b);
        let takerAtaAInfo = await getAccount(provider.connection, taker_ata_a);
        console.log("deal finalized succefully✅ ", tx);
        console.log("after finalizing deal amount of makerAta B having: ",makerAtaBInfo.amount);
        console.log("after finalizign deal amount of tkaerAta A having: ",takerAtaAInfo.amount);
        let makerAtaAInfo = await getAccount(provider.connection, maker_ata_a.address);
        let takerAtaBInfo = await getAccount(provider.connection, taker_ata_b.address);
        console.log("deal finalized succefully✅ ", tx);
        console.log("after finalizing deal amount of makerAta A having: ",makerAtaAInfo.amount);
        console.log("after finalizign deal amount of tkaerAta B having: ",takerAtaBInfo.amount);

    })

});
