import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { ArklyToken } from "../target/types/arkly_token";
import { PropertyVault } from "../target/types/property_vault";
import { PublicKey, Keypair, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { createMint, createAccount, mintTo, getAccount } from "@solana/spl-token";
import { assert, expect } from "chai";
import { BN } from "bn.js";

describe("Arkly Capital Smart Contracts", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const arklyTokenProgram = anchor.workspace.ArklyToken as Program<ArklyToken>;
  const propertyVaultProgram = anchor.workspace.PropertyVault as Program<PropertyVault>;

  let mint: PublicKey;
  let tokenomicsAccount: Keypair;
  let investorAccount: Keypair;
  let propertyAccount: Keypair;
  let adminKeypair: Keypair;

  before(async () => {
    // Initialize test accounts
    tokenomicsAccount = Keypair.generate();
    investorAccount = Keypair.generate();
    propertyAccount = Keypair.generate();
    adminKeypair = Keypair.generate();

    // Airdrop SOL to test accounts
    await provider.connection.requestAirdrop(adminKeypair.publicKey, 2 * LAMPORTS_PER_SOL);
    await provider.connection.requestAirdrop(investorAccount.publicKey, 2 * LAMPORTS_PER_SOL);
    
    // Wait for confirmation
    await new Promise(resolve => setTimeout(resolve, 1000));

    // Create ARKLY token mint
    mint = await createMint(
      provider.connection,
      adminKeypair,
      adminKeypair.publicKey,
      null,
      6 // 6 decimals
    );
  });

  describe("Arkly Token Program", () => {
    it("Initialize tokenomics", async () => {
      const totalSupply = new BN(100_000_000_000_000); // 100M tokens with 6 decimals
      const presalePrice = new BN(7500); // $0.0075 in smallest unit
      const hardcap = new BN(750_000_000_000); // $750k in smallest unit

      await arklyTokenProgram.methods
        .initializeTokenomics(totalSupply, presalePrice, hardcap)
        .accounts({
          tokenomics: tokenomicsAccount.publicKey,
          authority: adminKeypair.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([tokenomicsAccount, adminKeypair])
        .rpc();

      const tokenomicsData = await arklyTokenProgram.account.tokenomicsAllocations.fetch(
        tokenomicsAccount.publicKey
      );

      assert.equal(tokenomicsData.totalSupply.toString(), totalSupply.toString());
      assert.equal(tokenomicsData.presalePrice.toString(), presalePrice.toString());
      assert.equal(tokenomicsData.hardcap.toString(), hardcap.toString());
    });

    it("Purchase presale tokens", async () => {
      const purchaseAmount = new BN(1000_000_000); // $1,000
      const expectedTokens = new BN(133_333_333_333); // $1,000 / $0.0075 = 133,333.333333 tokens

      const investorTokenAccount = await createAccount(
        provider.connection,
        investorAccount,
        mint,
        investorAccount.publicKey
      );

      await arklyTokenProgram.methods
        .purchasePresale(purchaseAmount)
        .accounts({
          tokenomics: tokenomicsAccount.publicKey,
          investor: investorAccount.publicKey,
          investorTokenAccount: investorTokenAccount,
          mint: mint,
          authority: adminKeypair.publicKey,
          tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        })
        .signers([investorAccount])
        .rpc();

      const tokenomicsData = await arklyTokenProgram.account.tokenomicsAllocations.fetch(
        tokenomicsAccount.publicKey
      );

      assert.isTrue(tokenomicsData.presaleSold.gte(purchaseAmount));
    });

    it("Claim vested tokens", async () => {
      // Fast forward time for testing (in production, this would be actual time passage)
      await arklyTokenProgram.methods
        .claimVestedTokens()
        .accounts({
          tokenomics: tokenomicsAccount.publicKey,
          investor: investorAccount.publicKey,
          investorTokenAccount: await createAccount(
            provider.connection,
            investorAccount,
            mint,
            investorAccount.publicKey
          ),
          mint: mint,
          authority: adminKeypair.publicKey,
          tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        })
        .signers([investorAccount])
        .rpc();

      // Test passes if no error is thrown
      assert.isTrue(true);
    });
  });

  describe("Property Vault Program", () => {
    let propertyVaultAccount: Keypair;

    before(async () => {
      propertyVaultAccount = Keypair.generate();
    });

    it("Initialize property", async () => {
      const propertyValue = new BN(1_000_000_000_000); // $1M
      const expectedYield = new BN(8000); // 8% annual yield
      const tokenPrice = new BN(100_000_000); // $100 per token

      await propertyVaultProgram.methods
        .initializeProperty(
          "Luxury Apartment Complex",
          "Prime real estate in downtown area",
          propertyValue,
          expectedYield,
          tokenPrice
        )
        .accounts({
          property: propertyVaultAccount.publicKey,
          authority: adminKeypair.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([propertyVaultAccount, adminKeypair])
        .rpc();

      const propertyData = await propertyVaultProgram.account.property.fetch(
        propertyVaultAccount.publicKey
      );

      assert.equal(propertyData.name, "Luxury Apartment Complex");
      assert.equal(propertyData.totalValue.toString(), propertyValue.toString());
      assert.equal(propertyData.expectedYield.toString(), expectedYield.toString());
    });

    it("Purchase property tokens", async () => {
      const tokenAmount = new BN(10); // Buy 10 property tokens
      const paymentAmount = new BN(1_000_000_000); // $1,000

      await propertyVaultProgram.methods
        .purchasePropertyTokens(tokenAmount)
        .accounts({
          property: propertyVaultAccount.publicKey,
          investor: investorAccount.publicKey,
          authority: adminKeypair.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([investorAccount])
        .rpc();

      const propertyData = await propertyVaultProgram.account.property.fetch(
        propertyVaultAccount.publicKey
      );

      assert.isTrue(propertyData.tokensSold.gte(tokenAmount));
    });

    it("Distribute yield", async () => {
      const yieldAmount = new BN(80_000_000_000); // $80,000 annual yield

      await propertyVaultProgram.methods
        .distributeYield(yieldAmount)
        .accounts({
          property: propertyVaultAccount.publicKey,
          authority: adminKeypair.publicKey,
        })
        .signers([adminKeypair])
        .rpc();

      const propertyData = await propertyVaultProgram.account.property.fetch(
        propertyVaultAccount.publicKey
      );

      assert.isTrue(propertyData.totalYieldDistributed.gte(yieldAmount));
    });

    it("Claim yield", async () => {
      await propertyVaultProgram.methods
        .claimYield()
        .accounts({
          property: propertyVaultAccount.publicKey,
          investor: investorAccount.publicKey,
          authority: adminKeypair.publicKey,
        })
        .signers([investorAccount])
        .rpc();

      // Test passes if no error is thrown
      assert.isTrue(true);
    });
  });

  describe("Integration Tests", () => {
    it("Complete investment flow", async () => {
      // 1. Purchase ARKLY tokens in presale
      const presalePurchase = new BN(5000_000_000); // $5,000
      
      // 2. Use ARKLY tokens to purchase property tokens
      const propertyTokens = new BN(50); // 50 property tokens
      
      // 3. Claim yield over time
      // This would test the complete user journey from presale to yield generation
      
      assert.isTrue(true); // Placeholder for integration test
    });
  });
});
