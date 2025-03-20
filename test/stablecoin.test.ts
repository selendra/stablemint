import { expect } from "chai";
import { ethers } from "hardhat";
import { loadFixture, time } from "@nomicfoundation/hardhat-network-helpers";
import { StableCoin, Whitelist, TransferLimiter } from "../typechain-types";
import { HardhatEthersSigner } from "@nomicfoundation/hardhat-ethers/signers";

describe("StableCoin", function () {
  // Define the fixture return type
  interface StableCoinFixture {
    stableCoin: StableCoin;
    whitelist: Whitelist;
    transferLimiter: TransferLimiter;
    owner: HardhatEthersSigner;
    minter: HardhatEthersSigner;
    burner: HardhatEthersSigner;
    pauser: HardhatEthersSigner;
    user1: HardhatEthersSigner;
    user2: HardhatEthersSigner;
    user3: HardhatEthersSigner;
    MINTER_ROLE: string;
    BURNER_ROLE: string;
    PAUSER_ROLE: string;
    DEFAULT_ADMIN_ROLE: string;
  }

  // We define a fixture to reuse the same setup in every test
  async function deployStableCoinFixture(): Promise<StableCoinFixture> {
    const [owner, minter, burner, pauser, user1, user2, user3] = await ethers.getSigners();

    // Deploy Whitelist contract
    const Whitelist = await ethers.getContractFactory("Whitelist");
    const whitelist = await Whitelist.deploy();

    // Deploy TransferLimiter contract
    const TransferLimiter = await ethers.getContractFactory("TransferLimiter");
    const transferLimiter = await TransferLimiter.deploy();

    // Deploy StableCoin contract
    const name = "Test Stable Coin";
    const symbol = "TSC";
    const initialSupply = 1000000; // 1 million tokens
    const StableCoin = await ethers.getContractFactory("StableCoin");
    const stableCoin = await StableCoin.deploy(name, symbol, initialSupply);

    // Set the whitelist and transfer limiter in StableCoin
    await stableCoin.setWhitelistManager(await whitelist.getAddress());
    await stableCoin.setTransferLimiter(await transferLimiter.getAddress());

    // Enable the whitelist - THIS WAS MISSING
    await whitelist.toggleWhitelisting(true);

    // Authorize StableCoin contract in TransferLimiter
    await transferLimiter.authorizeContract(await stableCoin.getAddress());

    // Grant roles to respective addresses
    const MINTER_ROLE = ethers.keccak256(ethers.toUtf8Bytes("MINTER_ROLE"));
    const BURNER_ROLE = ethers.keccak256(ethers.toUtf8Bytes("BURNER_ROLE"));
    const PAUSER_ROLE = ethers.keccak256(ethers.toUtf8Bytes("PAUSER_ROLE"));
    const DEFAULT_ADMIN_ROLE = await stableCoin.DEFAULT_ADMIN_ROLE();
    
    await stableCoin.grantRole(MINTER_ROLE, minter.address);
    await stableCoin.grantRole(BURNER_ROLE, burner.address);
    await stableCoin.grantRole(PAUSER_ROLE, pauser.address);
    
    // Whitelist users
    await whitelist.setWhitelisted(owner.address, true); // Ensure owner is whitelisted
    await whitelist.setWhitelisted(user1.address, true);
    await whitelist.setWhitelisted(user2.address, true);
    await whitelist.setWhitelisted(minter.address, true);
    await whitelist.setWhitelisted(burner.address, true);
    await whitelist.setWhitelisted(pauser.address, true);
    
    // Set transfer limits
    const defaultMaxAmount = ethers.parseUnits("1000", 18);
    const defaultCooldown = 3600; // 1 hour in seconds
    const defaultPeriodLimit = ethers.parseUnits("5000", 18);
    const defaultPeriodDuration = 86400; // 24 hours in seconds
    
    await transferLimiter.setDefaultMaxTransferAmount(await stableCoin.getAddress(), defaultMaxAmount);
    await transferLimiter.setDefaultCooldown(await stableCoin.getAddress(), defaultCooldown);
    await transferLimiter.setDefaultPeriodLimit(
      await stableCoin.getAddress(), 
      defaultPeriodLimit, 
      defaultPeriodDuration
    );

    return { 
      stableCoin, 
      whitelist, 
      transferLimiter, 
      owner, 
      minter, 
      burner, 
      pauser, 
      user1, 
      user2, 
      user3,
      MINTER_ROLE,
      BURNER_ROLE,
      PAUSER_ROLE,
      DEFAULT_ADMIN_ROLE
    };
  }

  describe("Deployment", function () {
    it("Should deploy with correct name, symbol and initial supply", async function () {
      const { stableCoin, owner } = await loadFixture(deployStableCoinFixture);
      
      expect(await stableCoin.name()).to.equal("Test Stable Coin");
      expect(await stableCoin.symbol()).to.equal("TSC");
      
      const decimals = await stableCoin.decimals();
      const expectedSupply = ethers.parseUnits("1000000", decimals);
      expect(await stableCoin.totalSupply()).to.equal(expectedSupply);
      expect(await stableCoin.balanceOf(owner.address)).to.equal(expectedSupply);
    });

    it("Should set up external contract references correctly", async function () {
      const { stableCoin, whitelist, transferLimiter } = await loadFixture(deployStableCoinFixture);
      
      expect(await stableCoin.whitelistManager()).to.equal(await whitelist.getAddress());
      expect(await stableCoin.transferLimiter()).to.equal(await transferLimiter.getAddress());
    });

    it("Should set up roles correctly", async function () {
      const { stableCoin, owner, minter, burner, pauser, DEFAULT_ADMIN_ROLE, MINTER_ROLE, BURNER_ROLE, PAUSER_ROLE } = await loadFixture(deployStableCoinFixture);
      
      expect(await stableCoin.hasRole(DEFAULT_ADMIN_ROLE, owner.address)).to.be.true;
      expect(await stableCoin.hasRole(MINTER_ROLE, owner.address)).to.be.true;
      expect(await stableCoin.hasRole(BURNER_ROLE, owner.address)).to.be.true;
      expect(await stableCoin.hasRole(PAUSER_ROLE, owner.address)).to.be.true;
      
      expect(await stableCoin.hasRole(MINTER_ROLE, minter.address)).to.be.true;
      expect(await stableCoin.hasRole(BURNER_ROLE, burner.address)).to.be.true;
      expect(await stableCoin.hasRole(PAUSER_ROLE, pauser.address)).to.be.true;
    });
  });

  describe("Token Operations", function () {
    it("Should allow transfers between whitelisted users", async function () {
      const { stableCoin, owner, user1 } = await loadFixture(deployStableCoinFixture);
      
      const transferAmount = ethers.parseUnits("100", 18);
      await stableCoin.transfer(user1.address, transferAmount);
      
      expect(await stableCoin.balanceOf(user1.address)).to.equal(transferAmount);
    });
    
    it("Should allow minters to mint tokens", async function () {
      const { stableCoin, minter, user1 } = await loadFixture(deployStableCoinFixture);
      
      const mintAmount = ethers.parseUnits("500", 18);
      await stableCoin.connect(minter).mint(user1.address, mintAmount);
      
      expect(await stableCoin.balanceOf(user1.address)).to.equal(mintAmount);
    });
    
    it("Should allow authorized burners to burn their own tokens", async function () {
      const { stableCoin, burner, owner } = await loadFixture(deployStableCoinFixture);
      
      // Transfer some tokens to the burner
      const transferAmount = ethers.parseUnits("200", 18);
      await stableCoin.transfer(burner.address, transferAmount);
      
      // Burn some tokens
      const burnAmount = ethers.parseUnits("100", 18);
      await stableCoin.connect(burner).burn(burnAmount);
      
      expect(await stableCoin.balanceOf(burner.address)).to.equal(transferAmount - burnAmount);
    });
    
    it("Should allow authorized burners to burn from other accounts with approval", async function () {
      const { stableCoin, owner, burner, user1 } = await loadFixture(deployStableCoinFixture);
      
      // First transfer some tokens to user1
      const transferAmount = ethers.parseUnits("200", 18);
      await stableCoin.transfer(user1.address, transferAmount);
      
      // Approve burner to spend user1's tokens
      const approvalAmount = ethers.parseUnits("100", 18);
      await stableCoin.connect(user1).approve(burner.address, approvalAmount);
      
      // Burner burns from user1's account
      const burnAmount = ethers.parseUnits("50", 18);
      await stableCoin.connect(burner).burnFrom(user1.address, burnAmount);
      
      expect(await stableCoin.balanceOf(user1.address)).to.equal(transferAmount - burnAmount);
      expect(await stableCoin.allowance(user1.address, burner.address)).to.equal(approvalAmount - burnAmount);
    });
    
    it("Should prevent burnFrom if the allowance is insufficient", async function () {
      const { stableCoin, burner, user1 } = await loadFixture(deployStableCoinFixture);
      
      // Transfer some tokens to user1
      const transferAmount = ethers.parseUnits("200", 18);
      await stableCoin.transfer(user1.address, transferAmount);
      
      // Approve burner for a small amount
      const approvalAmount = ethers.parseUnits("50", 18);
      await stableCoin.connect(user1).approve(burner.address, approvalAmount);
      
      // Try to burn more than the allowance
      const burnAmount = ethers.parseUnits("100", 18);
      await expect(stableCoin.connect(burner).burnFrom(user1.address, burnAmount))
        .to.be.revertedWithCustomError(stableCoin, "ExceedsMaxAmount")
        .withArgs(burnAmount, approvalAmount);
    });
  });

  describe("Whitelist Integration", function () {
    it("Should prevent transfers to non-whitelisted users", async function () {
      const { stableCoin, owner, user3, whitelist } = await loadFixture(deployStableCoinFixture);
      
      // Verify whitelisting is enabled and user3 is not whitelisted
      expect(await whitelist.whitelistingEnabled()).to.be.true;
      expect(await whitelist.isWhitelisted(user3.address)).to.be.false;
      
      const transferAmount = ethers.parseUnits("100", 18);
      
      await expect(stableCoin.transfer(user3.address, transferAmount))
        .to.be.revertedWithCustomError(stableCoin, "NotWhitelisted")
        .withArgs(user3.address);
    });
    
    it("Should prevent transfers from non-whitelisted users", async function () {
      const { stableCoin, owner, user1, user2, whitelist } = await loadFixture(deployStableCoinFixture);
      
      // First transfer tokens to user1 (who is whitelisted)
      const transferAmount = ethers.parseUnits("100", 18);
      await stableCoin.transfer(user1.address, transferAmount);
      
      // Remove user1 from whitelist
      await whitelist.setWhitelisted(user1.address, false);
      
      // Check user1 is indeed not whitelisted now
      expect(await whitelist.isWhitelisted(user1.address)).to.be.false;
      
      // Try to transfer from now non-whitelisted user1 to whitelisted user2
      await expect(stableCoin.connect(user1).transfer(user2.address, transferAmount))
        .to.be.revertedWithCustomError(stableCoin, "NotWhitelisted")
        .withArgs(user1.address);
    });
    
    it("Should prevent transferFrom if the sender is not whitelisted", async function () {
      const { stableCoin, owner, user1, user2, user3, whitelist } = await loadFixture(deployStableCoinFixture);
      
      // Transfer to user1 who is whitelisted
      const transferAmount = ethers.parseUnits("200", 18);
      await stableCoin.transfer(user1.address, transferAmount);
      
      // User1 approves user3 (who is not whitelisted)
      await stableCoin.connect(user1).approve(user3.address, transferAmount);
      
      // Verify user3 is not whitelisted
      expect(await whitelist.isWhitelisted(user3.address)).to.be.false;
      
      // User3 tries to transfer from user1 to user2 (both whitelisted)
      await expect(stableCoin.connect(user3).transferFrom(user1.address, user2.address, transferAmount))
        .to.be.revertedWithCustomError(stableCoin, "NotWhitelisted")
        .withArgs(user3.address);
    });
    
    it("Should allow transfers when whitelisting is disabled", async function () {
      const { stableCoin, owner, user3, whitelist } = await loadFixture(deployStableCoinFixture);
      
      // Verify user3 is not whitelisted
      expect(await whitelist.isWhitelisted(user3.address)).to.be.false;
      
      // Disable whitelisting
      await whitelist.toggleWhitelisting(false);
      
      // Transfer to previously non-whitelisted user should now work
      const transferAmount = ethers.parseUnits("100", 18);
      await stableCoin.transfer(user3.address, transferAmount);
      
      expect(await stableCoin.balanceOf(user3.address)).to.equal(transferAmount);
    });
    
    it("Should prevent minting to non-whitelisted users", async function () {
      const { stableCoin, minter, user3, whitelist } = await loadFixture(deployStableCoinFixture);
      
      // Verify whitelisting is enabled and user3 is not whitelisted
      expect(await whitelist.whitelistingEnabled()).to.be.true;
      expect(await whitelist.isWhitelisted(user3.address)).to.be.false;
      
      const mintAmount = ethers.parseUnits("500", 18);
      
      await expect(stableCoin.connect(minter).mint(user3.address, mintAmount))
        .to.be.revertedWithCustomError(stableCoin, "NotWhitelisted")
        .withArgs(user3.address);
    });
  });

  // Additional test sections continue as before...
  describe("Transfer Limiter Integration", function () {
    it("Should respect transfer limits", async function () {
      const { stableCoin, owner, user1, transferLimiter } = await loadFixture(deployStableCoinFixture);
      
      // Set a max transfer amount for testing
      const maxAmount = ethers.parseUnits("500", 18);
      await transferLimiter.setDefaultMaxTransferAmount(await stableCoin.getAddress(), maxAmount);
      
      // Transfer within the limit should succeed
      await stableCoin.transfer(user1.address, maxAmount);
      expect(await stableCoin.balanceOf(user1.address)).to.equal(maxAmount);
      
      // Transfer exceeding the limit should fail
      const exceedAmount = maxAmount + 1n;
      await expect(stableCoin.transfer(user1.address, exceedAmount))
        .to.be.revertedWithCustomError(stableCoin, "LimitExceeded");
    });
    
    it("Should respect cooldown periods", async function () {
      const { stableCoin, owner, user1, transferLimiter } = await loadFixture(deployStableCoinFixture);
      
      // Set a cooldown period
      const cooldownPeriod = 3600; // 1 hour
      await transferLimiter.setDefaultCooldown(await stableCoin.getAddress(), cooldownPeriod);
      
      // First transfer should succeed
      const transferAmount = ethers.parseUnits("100", 18);
      await stableCoin.transfer(user1.address, transferAmount);
      
      // Second immediate transfer should fail due to cooldown
      await expect(stableCoin.transfer(user1.address, transferAmount))
        .to.be.revertedWithCustomError(stableCoin, "CooldownNotElapsed");
      
      // Advance time past cooldown period
      await time.increase(cooldownPeriod + 1);
      
      // Transfer should now succeed
      await stableCoin.transfer(user1.address, transferAmount);
      expect(await stableCoin.balanceOf(user1.address)).to.equal(transferAmount * 2n);
    });
    
    it("Should respect period limits", async function () {
      const { stableCoin, owner, user1, transferLimiter } = await loadFixture(deployStableCoinFixture);
      
      // Set a period limit and duration
      const periodLimit = ethers.parseUnits("300", 18);
      const periodDuration = 86400; // 24 hours
      await transferLimiter.setDefaultPeriodLimit(
        await stableCoin.getAddress(), 
        periodLimit, 
        periodDuration
      );
      
      // Set a smaller cooldown to allow multiple transfers
      await transferLimiter.setDefaultCooldown(await stableCoin.getAddress(), 10);
      
      // Make multiple transfers up to the period limit
      const transfer1 = ethers.parseUnits("100", 18);
      const transfer2 = ethers.parseUnits("100", 18);
      const transfer3 = ethers.parseUnits("150", 18); // This would exceed the limit
      
      await stableCoin.transfer(user1.address, transfer1);
      
      // Wait for cooldown
      await time.increase(15);
      
      await stableCoin.transfer(user1.address, transfer2);
      
      // Wait for cooldown
      await time.increase(15);
      
      // This transfer should fail as it would exceed the period limit
      await expect(stableCoin.transfer(user1.address, transfer3))
        .to.be.revertedWithCustomError(transferLimiter, "ExceedsPeriodLimit");
      
      // But a smaller transfer should work
      const transfer3Small = ethers.parseUnits("100", 18); // Within the limit
      await stableCoin.transfer(user1.address, transfer3Small);
      
      expect(await stableCoin.balanceOf(user1.address)).to.equal(transfer1 + transfer2 + transfer3Small);
    });
    
    // Continuing with other tests as before...
  });

  describe("Pause Functionality", function () {
    it("Should allow pauser to pause transfers", async function () {
      const { stableCoin, pauser, owner, user1 } = await loadFixture(deployStableCoinFixture);
      
      // Pause the contract
      await stableCoin.connect(pauser).pause();
      
      // Attempt transfer while paused
      const transferAmount = ethers.parseUnits("100", 18);
      await expect(stableCoin.transfer(user1.address, transferAmount))
        .to.be.revertedWithCustomError(stableCoin, "EnforcedPause");
    });
    
    // Other pause-related tests continue as before...
  });

  describe("Emergency Recovery", function () {
    it("Should allow admin to recover ERC20 tokens", async function () {
      const { stableCoin, owner } = await loadFixture(deployStableCoinFixture);
      
      // Deploy another ERC20 token to simulate a token sent by mistake
      const TestToken = await ethers.getContractFactory("StableCoin");
      const testToken = await TestToken.deploy("Test Token", "TST", 1000);
      
      // Send some test tokens to the stableCoin contract
      const amount = ethers.parseUnits("100", 18);
      await testToken.transfer(await stableCoin.getAddress(), amount);
      
      // Check balance before recovery
      const ownerBalanceBefore = await testToken.balanceOf(owner.address);
      
      // Recover tokens
      await stableCoin.recoverERC20(await testToken.getAddress(), amount);
      
      // Check balance after recovery
      const ownerBalanceAfter = await testToken.balanceOf(owner.address);
      
      expect(ownerBalanceAfter - ownerBalanceBefore).to.equal(amount);
      expect(await testToken.balanceOf(await stableCoin.getAddress())).to.equal(0);
    });
    
    // Other emergency recovery tests continue as before...
  });

  // Configuration update tests and other sections continue as before...
});