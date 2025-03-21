import { expect } from "chai";
import { ethers } from "hardhat";
import { HardhatEthersSigner } from "@nomicfoundation/hardhat-ethers/signers";
import { StableCoin, Whitelist, TransferLimiter } from "../typechain-types";

describe("StableCoin", function () {
  let stableCoin: StableCoin;
  let whitelist: Whitelist;
  let transferLimiter: TransferLimiter;
  let admin: HardhatEthersSigner, minter: HardhatEthersSigner, burner: HardhatEthersSigner, 
      pauser: HardhatEthersSigner, user1: HardhatEthersSigner, user2: HardhatEthersSigner;
  
  const name = "Optimized Stable Coin";
  const symbol = "OSC";
  const initialSupply = 1000000n; // 1 million
  
  let MINTER_ROLE: string;
  let BURNER_ROLE: string;
  let PAUSER_ROLE: string;
  
  beforeEach(async function () {
    [admin, minter, burner, pauser, user1, user2] = await ethers.getSigners();
    
    // Deploy Whitelist
    const WhitelistFactory = await ethers.getContractFactory("Whitelist");
    whitelist = await WhitelistFactory.deploy(true) as Whitelist;
    await whitelist.waitForDeployment();
    
    // Deploy TransferLimiter
    const TransferLimiterFactory = await ethers.getContractFactory("TransferLimiter");
    transferLimiter = await TransferLimiterFactory.deploy() as TransferLimiter;
    await transferLimiter.waitForDeployment();
    
    // Deploy StableCoin
    const StableCoinFactory = await ethers.getContractFactory("StableCoin");
    stableCoin = await StableCoinFactory.deploy(
      name,
      symbol,
      initialSupply,
      await whitelist.getAddress(),
      await transferLimiter.getAddress()
    ) as StableCoin;
    await stableCoin.waitForDeployment();
    
    // Get StableCoin address
    const stableCoinAddress = await stableCoin.getAddress();
    
    // Calculate role hashes
    MINTER_ROLE = ethers.keccak256(ethers.toUtf8Bytes("MINTER_ROLE"));
    BURNER_ROLE = ethers.keccak256(ethers.toUtf8Bytes("BURNER_ROLE"));
    PAUSER_ROLE = ethers.keccak256(ethers.toUtf8Bytes("PAUSER_ROLE"));
    
    // Configure supporting contracts
    
    // IMPORTANT: Explicitly authorize the StableCoin in TransferLimiter
    const CONTRACT_ROLE = await transferLimiter.CONTRACT_ROLE();
    
    // Check if it's already authorized and authorize if not
    if (!(await transferLimiter.hasRole(CONTRACT_ROLE, stableCoinAddress))) {
      // Force authorize the contract
      await transferLimiter.authorizeContract(stableCoinAddress);
      console.log("Explicitly authorized StableCoin in TransferLimiter");
    }
    
    // Set default limits in TransferLimiter
    const limitConfig = {
      maxTransferAmount: ethers.parseUnits("10000", 18),
      cooldownPeriod: 60n,
      periodLimit: ethers.parseUnits("50000", 18),
      periodDuration: 86400n
    };
    await transferLimiter.setAllDefaultLimits(stableCoinAddress, limitConfig);
    
    // Whitelist test users AND the burner account
    await whitelist.batchSetWhitelisted([
      admin.address, 
      minter.address, 
      burner.address, 
      pauser.address, 
      user1.address, 
      user2.address
    ], true);
    
    // Exempt admin from transfer limits
    await transferLimiter.setExemption(stableCoinAddress, admin.address, true);
    
    // Setup roles
    await stableCoin.grantRole(MINTER_ROLE, minter.address);
    await stableCoin.grantRole(BURNER_ROLE, burner.address);
    await stableCoin.grantRole(PAUSER_ROLE, pauser.address);
  });
  
  describe("Initialization", function () {
    it("should initialize with correct name and symbol", async function () {
      expect(await stableCoin.name()).to.equal(name);
      expect(await stableCoin.symbol()).to.equal(symbol);
    });
    
    it("should mint initial supply to deployer", async function () {
      expect(await stableCoin.balanceOf(admin.address)).to.equal(ethers.parseUnits(initialSupply.toString(), 18));
    });
    
    it("should set correct external service addresses", async function () {
      expect(await stableCoin.whitelistManager()).to.equal(await whitelist.getAddress());
      expect(await stableCoin.transferLimiter()).to.equal(await transferLimiter.getAddress());
    });
    
    it("should enable whitelist and limit checks by default", async function () {
      expect(await stableCoin.whitelistChecksEnabled()).to.be.true;
      expect(await stableCoin.limitChecksEnabled()).to.be.true;
    });
  });
  
  describe("Role-based Access Control", function () {
    it("should allow minter to mint tokens", async function () {
      const mintAmount = ethers.parseUnits("1000", 18);
      await stableCoin.connect(minter).mint(user1.address, mintAmount);
      expect(await stableCoin.balanceOf(user1.address)).to.equal(mintAmount);
    });
    
    it("should prevent non-minter from minting tokens", async function () {
      const mintAmount = ethers.parseUnits("1000", 18);
      await expect(
        stableCoin.connect(user1).mint(user1.address, mintAmount)
      ).to.be.reverted;
    });
    
    it("should allow burner to burn tokens", async function () {
      // First transfer some tokens to the burner
      const burnAmount = ethers.parseUnits("1000", 18);
      await stableCoin.transfer(burner.address, burnAmount);
      
      // Then burn them
      await stableCoin.connect(burner).burn(burnAmount);
      expect(await stableCoin.balanceOf(burner.address)).to.equal(0n);
    });
    
    it("should allow pauser to pause and unpause", async function () {
      await stableCoin.connect(pauser).pause();
      expect(await stableCoin.paused()).to.be.true;
      
      await stableCoin.connect(pauser).unpause();
      expect(await stableCoin.paused()).to.be.false;
    });
  });
  
  describe("Transfer with Whitelist", function () {
    it("should allow transfers between whitelisted addresses", async function () {
      const transferAmount = ethers.parseUnits("100", 18);
      await stableCoin.transfer(user1.address, transferAmount);
      
      expect(await stableCoin.balanceOf(user1.address)).to.equal(transferAmount);
      
      await stableCoin.connect(user1).transfer(user2.address, transferAmount);
      expect(await stableCoin.balanceOf(user2.address)).to.equal(transferAmount);
    });
    
    it("should prevent transfers to non-whitelisted addresses", async function () {
      const nonWhitelistedUser = user2;
      await whitelist.setWhitelisted(nonWhitelistedUser.address, false);
      
      const transferAmount = ethers.parseUnits("100", 18);
      await expect(
        stableCoin.transfer(nonWhitelistedUser.address, transferAmount)
      ).to.be.revertedWithCustomError(stableCoin, "NotWhitelisted");
    });
    
    it("should allow transfers when whitelist checks are disabled", async function () {
      // Remove user2 from whitelist
      await whitelist.setWhitelisted(user2.address, false);
      
      // Disable whitelist checks
      await stableCoin.updateConfig(true, false);
      
      // Now transfer should succeed
      const transferAmount = ethers.parseUnits("100", 18);
      await stableCoin.transfer(user2.address, transferAmount);
      expect(await stableCoin.balanceOf(user2.address)).to.equal(transferAmount);
    });
  });
  
  describe("Transfer with Limits", function () {
    beforeEach(async function () {
      // Transfer some tokens to user1 for testing
      const initialAmount = ethers.parseUnits("20000", 18);
      await stableCoin.transfer(user1.address, initialAmount);
    });
    
    it("should prevent transfers exceeding single transfer limit", async function () {
      const maxLimit = ethers.parseUnits("10000", 18);
      const exceedingAmount = maxLimit + 1n;
      
      await expect(
        stableCoin.connect(user1).transfer(user2.address, exceedingAmount)
      ).to.be.revertedWithCustomError(stableCoin, "LimitExceeded");
    });
    it("should prevent transfers that would exceed period limit", async function () {
      const stableCoinAddress = await stableCoin.getAddress();
      
      // Temporarily increase the single transfer limit for this test
      await transferLimiter.setDefaultMaxTransferAmount(
        stableCoinAddress, 
        ethers.parseUnits("100000", 18) // Much higher than the period limit
      );
      
      // First transfer at the limit
      const periodLimit = ethers.parseUnits("50000", 18);
      
      // Get current user1 balance
      const user1Balance = await stableCoin.balanceOf(user1.address);
      
      // Send more funds to user1 if needed
      if (user1Balance < periodLimit) {
        await stableCoin.transfer(user1.address, periodLimit - user1Balance);
      }
      
      // Now make a transfer at the period limit
      await stableCoin.connect(user1).transfer(user2.address, periodLimit);
      
      // Try to make another transfer - should fail due to period limit
      const smallAmount = ethers.parseUnits("1", 18);
      await stableCoin.transfer(user1.address, smallAmount); // Give user1 some more tokens
      
      await expect(
        stableCoin.connect(user1).transfer(user2.address, smallAmount)
      ).to.be.reverted; // Will revert due to period limit in TransferLimiter
      
      // Restore the original limit if needed
      await transferLimiter.setDefaultMaxTransferAmount(
        stableCoinAddress, 
        ethers.parseUnits("10000", 18)
      );
    });
    
    it("should allow transfers when limit checks are disabled", async function () {
      // Disable limit checks
      await stableCoin.updateConfig(false, true);
      
      // Now transfer exceeding limit should succeed
      const largeAmount = ethers.parseUnits("15000", 18);
      
      // Get current user1 balance
      const user1Balance = await stableCoin.balanceOf(user1.address);
      
      // Send more funds to user1 if needed
      if (user1Balance < largeAmount) {
        await stableCoin.transfer(user1.address, largeAmount - user1Balance);
      }
      
      await stableCoin.connect(user1).transfer(user2.address, largeAmount);
      expect(await stableCoin.balanceOf(user2.address)).to.equal(largeAmount);
    });
  });
  
  describe("Pause Functionality", function () {
    it("should prevent transfers when paused", async function () {
      await stableCoin.connect(pauser).pause();
      
      const transferAmount = ethers.parseUnits("100", 18);
      await expect(
        stableCoin.transfer(user1.address, transferAmount)
      ).to.be.reverted; // Reverts with Pausable's error
    });
    
    it("should allow transfers after unpausing", async function () {
      await stableCoin.connect(pauser).pause();
      await stableCoin.connect(pauser).unpause();
      
      const transferAmount = ethers.parseUnits("100", 18);
      await stableCoin.transfer(user1.address, transferAmount);
      expect(await stableCoin.balanceOf(user1.address)).to.equal(transferAmount);
    });
  });
  
  describe("External Service Updates", function () {
    it("should allow admin to update whitelist manager", async function () {
      // Deploy a new whitelist
      const WhitelistFactory = await ethers.getContractFactory("Whitelist");
      const newWhitelist = await WhitelistFactory.deploy(true) as Whitelist;
      await newWhitelist.waitForDeployment();
      
      // Update whitelist manager
      await stableCoin.setWhitelistManager(await newWhitelist.getAddress());
      expect(await stableCoin.whitelistManager()).to.equal(await newWhitelist.getAddress());
    });
    
    it("should allow admin to update transfer limiter", async function () {
      // Deploy a new transfer limiter
      const TransferLimiterFactory = await ethers.getContractFactory("TransferLimiter");
      const newTransferLimiter = await TransferLimiterFactory.deploy() as TransferLimiter;
      await newTransferLimiter.waitForDeployment();
      
      // Update transfer limiter
      await stableCoin.setTransferLimiter(await newTransferLimiter.getAddress());
      expect(await stableCoin.transferLimiter()).to.equal(await newTransferLimiter.getAddress());
    });
  });
  
  describe("Events", function () {
    it("should emit TokensMinted event when minting", async function () {
      const mintAmount = ethers.parseUnits("1000", 18);
      await expect(stableCoin.connect(minter).mint(user1.address, mintAmount))
        .to.emit(stableCoin, "TokensMinted")
        .withArgs(user1.address, mintAmount);
    });
    
    it("should emit TokensBurned event when burning", async function () {
      // First transfer some tokens to the burner
      const burnAmount = ethers.parseUnits("1000", 18);
      await stableCoin.transfer(burner.address, burnAmount);
      
      // Then burn them
      await expect(stableCoin.connect(burner).burn(burnAmount))
        .to.emit(stableCoin, "TokensBurned")
        .withArgs(burner.address, burnAmount);
    });
    
    it("should emit ConfigUpdated event when config changes", async function () {
      await expect(stableCoin.updateConfig(false, false))
        .to.emit(stableCoin, "ConfigUpdated")
        .withArgs(false, false);
    });
  });
});