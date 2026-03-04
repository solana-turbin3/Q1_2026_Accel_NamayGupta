import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { NftStakingCore } from "../target/types/nft_staking_core";
import { SystemProgram } from "@solana/web3.js";
import { MPL_CORE_PROGRAM_ID } from "@metaplex-foundation/mpl-core";
import { ASSOCIATED_TOKEN_PROGRAM_ID, getAssociatedTokenAddressSync, TOKEN_PROGRAM_ID } from "@solana/spl-token";

const MILLISECONDS_PER_DAY = 86400000;
const POINTS_PER_STAKED_NFT_PER_DAY = 10_000_000;
const FREEZE_PERIOD_IN_DAYS = 7;
const TIME_TRAVEL_IN_DAYS = 8;

describe("nft-staking-core", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.nftStakingCore as Program<NftStakingCore>;

  const collectionKeypair = anchor.web3.Keypair.generate();

  const updateAuthority = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("update_authority"), collectionKeypair.publicKey.toBuffer()],
    program.programId
  )[0];


  const nft1Keypair = anchor.web3.Keypair.generate();

  const nft2Keypair = anchor.web3.Keypair.generate();
  // Receiver for transfer test
  const receiver = anchor.web3.Keypair.generate();

  const config = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("config"), collectionKeypair.publicKey.toBuffer()],
    program.programId
  )[0];

  const rewardsMint = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("rewards"), config.toBuffer()],
    program.programId
  )[0];

  const oracle = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("oracle")],
    program.programId
  )[0];

  const vault = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("vault"), oracle.toBuffer()],
    program.programId
  )[0];


  let dayOffset = 0;

  /**
   * Helper function to advance time with surfnet_timeTravel RPC method
   * @param params - Time travel params (absoluteEpoch, absoluteSlot, or absoluteTimestamp)
   */
  async function advanceTime(params: { absoluteEpoch?: number; absoluteSlot?: number; absoluteTimestamp?: number }): Promise<void> {
    const rpcResponse = await fetch(provider.connection.rpcEndpoint, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        jsonrpc: "2.0",
        id: 1,
        method: "surfnet_timeTravel",
        params: [params],
      }),
    });

    const result = await rpcResponse.json() as { error?: any; result?: any };
    if (result.error) {
      throw new Error(`Time travel failed: ${JSON.stringify(result.error)}`);
    }

    await new Promise((resolve) => setTimeout(resolve, 1000));
  }

  // ==================== SETUP ====================

  it("Create an oracle", async () => {
    const tx = await program.methods.createOracle().accountsPartial({
      payer: provider.wallet.publicKey,
      oracle,
      vault,
      systemProgram: SystemProgram.programId,
    }).rpc();
    console.log("\nYour transaction signature", tx);
  });

  it("Create a collection", async () => {
    const collectionName = "Test Collection";
    const collectionUri = "https://example.com/collection";
    const tx = await program.methods.createCollection(collectionName, collectionUri)
      .accountsPartial({
        payer: provider.wallet.publicKey,
        oracleAccount: oracle,
        collection: collectionKeypair.publicKey,
        updateAuthority,
        systemProgram: SystemProgram.programId,
        mplCoreProgram: MPL_CORE_PROGRAM_ID,
      })
      .signers([collectionKeypair])
      .rpc();
    console.log("\nYour transaction signature", tx);
    console.log("Collection address", collectionKeypair.publicKey.toBase58());
  });

  it("Mint NFT #1", async () => {
    const tx = await program.methods.mintNft("Test NFT 1", "https://example.com/nft1")
      .accountsPartial({
        user: provider.wallet.publicKey,
        nft: nft1Keypair.publicKey,
        collection: collectionKeypair.publicKey,
        updateAuthority,
        systemProgram: SystemProgram.programId,
        mplCoreProgram: MPL_CORE_PROGRAM_ID,
      })
      .signers([nft1Keypair])
      .rpc();
    console.log("\nNFT #1 address", nft1Keypair.publicKey.toBase58());
  });

  it("Mint NFT #2", async () => {
    const tx = await program.methods.mintNft("Test NFT 2", "https://example.com/nft2")
      .accountsPartial({
        user: provider.wallet.publicKey,
        nft: nft2Keypair.publicKey,
        collection: collectionKeypair.publicKey,
        updateAuthority,
        systemProgram: SystemProgram.programId,
        mplCoreProgram: MPL_CORE_PROGRAM_ID,
      })
      .signers([nft2Keypair])
      .rpc();
    console.log("\nNFT #2 address", nft2Keypair.publicKey.toBase58());
  });

  it("Initialize stake config", async () => {
    const tx = await program.methods.initializeConfig(POINTS_PER_STAKED_NFT_PER_DAY, FREEZE_PERIOD_IN_DAYS)
      .accountsPartial({
        admin: provider.wallet.publicKey,
        collection: collectionKeypair.publicKey,
        updateAuthority,
        config,
        rewardsMint,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();
    console.log("\nYour transaction signature", tx);
    console.log("Config address", config.toBase58());
    console.log("Points per staked NFT per day", POINTS_PER_STAKED_NFT_PER_DAY);
    console.log("Freeze period in days", FREEZE_PERIOD_IN_DAYS);
    console.log("Rewards mint address", rewardsMint.toBase58());
  });

  it("Stake NFT #1", async () => {
    await program.methods.stake()
      .accountsPartial({
        user: provider.wallet.publicKey,
        updateAuthority,
        config,
        nft: nft1Keypair.publicKey,
        collection: collectionKeypair.publicKey,
        systemProgram: SystemProgram.programId,
        mplCoreProgram: MPL_CORE_PROGRAM_ID,
      })
      .rpc();
    console.log("\nStaked NFT #1");
  });

  it("Stake NFT #2", async () => {
    await program.methods.stake()
      .accountsPartial({
        user: provider.wallet.publicKey,
        updateAuthority,
        config,
        nft: nft2Keypair.publicKey,
        collection: collectionKeypair.publicKey,
        systemProgram: SystemProgram.programId,
        mplCoreProgram: MPL_CORE_PROGRAM_ID,
      })
      .rpc();
    console.log("\nStaked NFT #2");
  });

  it("Time travel +8 days", async () => {
    dayOffset += TIME_TRAVEL_IN_DAYS;
    await advanceTime({ absoluteTimestamp: Date.now() + dayOffset * MILLISECONDS_PER_DAY });
    console.log("\nTime traveled, total offset:", dayOffset, "days");
  });

  it("Claim rewards (NFT #1)", async () => {
    const userRewardsAta = getAssociatedTokenAddressSync(rewardsMint, provider.wallet.publicKey, false, TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID);
    await program.methods.claimRewards().accountsPartial({
      user: provider.wallet.publicKey,
      updateAuthority,
      config,
      rewardsMint,
      userRewardsAta,
      nft: nft1Keypair.publicKey,
      collection: collectionKeypair.publicKey,
      mplCoreProgram: MPL_CORE_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
    }).rpc();
    const balance = (await provider.connection.getTokenAccountBalance(userRewardsAta)).value.uiAmount;
    console.log("\nClaimed rewards for NFT #1, balance:", balance);
  });

  it("Time travel +8 more days", async () => {
    dayOffset += TIME_TRAVEL_IN_DAYS;
    await advanceTime({ absoluteTimestamp: Date.now() + dayOffset * MILLISECONDS_PER_DAY });
    console.log("\nTime traveled, total offset:", dayOffset, "days");
  });

  xit("Unstake NFT #1", async () => {
    const userRewardsAta = getAssociatedTokenAddressSync(rewardsMint, provider.wallet.publicKey, false, TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID);
    await program.methods.unstake()
      .accountsPartial({
        user: provider.wallet.publicKey,
        updateAuthority,
        config,
        rewardsMint,
        userRewardsAta,
        nft: nft1Keypair.publicKey,
        collection: collectionKeypair.publicKey,
        mplCoreProgram: MPL_CORE_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      })
      .rpc();
    const balance = (await provider.connection.getTokenAccountBalance(userRewardsAta)).value.uiAmount;
    console.log("\nUnstaked NFT #1, rewards balance:", balance);
  });

  it("Burn staked NFT #2", async () => {
    const userRewardsAta = getAssociatedTokenAddressSync(rewardsMint, provider.wallet.publicKey, false, TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID);
    await program.methods.burnStakedNft().accountsPartial({
      user: provider.wallet.publicKey,
      updateAuthority,
      config,
      rewardsMint,
      userRewardsAta,
      nft: nft2Keypair.publicKey,
      collection: collectionKeypair.publicKey,
      mplCoreProgram: MPL_CORE_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
    }).rpc();
    const balance = (await provider.connection.getTokenAccountBalance(userRewardsAta)).value.uiAmount;
    console.log("\nBurned NFT #2, rewards balance:", balance);
  });

  it("Time travel to not allowed time", async () => {
    dayOffset += 1;
    const target = new Date(Date.now() + dayOffset * MILLISECONDS_PER_DAY);
    target.setUTCHours(20, 0, 0, 0);
    await advanceTime({ absoluteTimestamp: target.getTime() });
    console.log("\nTraveled to 8PM UTC (outside transfer window)");
  });

  it("Update oracle", async () => {
    const tx = await program.methods.updateValidationOracle()
      .accountsPartial({
        signer: provider.wallet.publicKey,
        oracle,
        vault,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const oracleData = await program.account.oracle.fetch(oracle);
    console.log("\nUpdated Oracle", JSON.stringify(oracleData.validation));
  });

  it("Time travel to allowed time", async () => {
    dayOffset += 1;
    const target = new Date(Date.now() + dayOffset * MILLISECONDS_PER_DAY);
    target.setUTCHours(10, 0, 0, 0);
    await advanceTime({ absoluteTimestamp: target.getTime() });
    console.log("\nTraveled to 10AM UTC (inside transfer window)");
  });

  it("Transfer NFT #1 to new owner", async () => {
    const tx = await program.methods.transfer()
      .accountsPartial({
        owner: provider.wallet.publicKey,
        updateAuthority,
        nft: nft1Keypair.publicKey,
        collection: collectionKeypair.publicKey,
        receiver: receiver.publicKey,
        oracle,
        mplCoreProgram: MPL_CORE_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    console.log("\nTransferred NFT #1 to", receiver.publicKey.toBase58());
  });

});
