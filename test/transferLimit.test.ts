const { expect } = require("chai");
const { ethers } = require("hardhat");
const { loadFixture, time } = require("@nomicfoundation/hardhat-network-helpers");

describe("TransferLimiter", function () {
  // Test fixture to deploy the contract and set up test accounts
  async function deployTransferLimiterFixture() {
    // Get signers
    const [owner, admin, limitManager, user1, user2, tokenAddress, contractAddress] = await ethers.getSigners();

    // Deploy the contract
    const TransferLimiter = await ethers.getContractFactory("TransferLimiter");
    const transferLimiter = await TransferLimiter.deploy();

    // Get role hashes
    const ADMIN_ROLE = await transferLimiter.ADMIN_ROLE();
    const CONTRACT_ROLE = await transferLimiter.CONTRACT_ROLE();
    const LIMIT_MANAGER_ROLE = await transferLimiter.LIMIT_MANAGER_ROLE();
    const DEFAULT_ADMIN_ROLE = await transferLimiter.DEFAULT_ADMIN_ROLE();

    return { 
      transferLimiter, 
      owner, 
      admin, 
      limitManager, 
      user1, 
      user2, 
      tokenAddress, 
      contractAddress,
      ADMIN_ROLE,
      CONTRACT_ROLE,
      LIMIT_MANAGER_ROLE,
      DEFAULT_ADMIN_ROLE
    };
  }

  describe("Deployment", function () {
    it("Should set the right owner roles", async function () {
      const { transferLimiter, owner, DEFAULT_ADMIN_ROLE, ADMIN_ROLE, LIMIT_MANAGER_ROLE } = await loadFixture(deployTransferLimiterFixture);
      
      expect(await transferLimiter.hasRole(DEFAULT_ADMIN_ROLE, owner.address)).to.be.true;
      expect(await transferLimiter.hasRole(ADMIN_ROLE, owner.address)).to.be.true;
      expect(await transferLimiter.hasRole(LIMIT_MANAGER_ROLE, owner.address)).to.be.true;
    });
  });

  describe("Role Management", function () {
    it("Should allow adding a limit manager", async function () {
      const { transferLimiter, owner, limitManager, LIMIT_MANAGER_ROLE } = await loadFixture(deployTransferLimiterFixture);
      
      await transferLimiter.addLimitManager(limitManager.address);
      expect(await transferLimiter.hasRole(LIMIT_MANAGER_ROLE, limitManager.address)).to.be.true;
    });

    it("Should allow removing a limit manager", async function () {
      const { transferLimiter, owner, limitManager, LIMIT_MANAGER_ROLE } = await loadFixture(deployTransferLimiterFixture);
      
      await transferLimiter.addLimitManager(limitManager.address);
      expect(await transferLimiter.hasRole(LIMIT_MANAGER_ROLE, limitManager.address)).to.be.true;
      
      await transferLimiter.removeLimitManager(limitManager.address);
      expect(await transferLimiter.hasRole(LIMIT_MANAGER_ROLE, limitManager.address)).to.be.false;
    });

    it("Should allow authorizing a contract", async function () {
      const { transferLimiter, contractAddress, CONTRACT_ROLE } = await loadFixture(deployTransferLimiterFixture);
      
      await transferLimiter.authorizeContract(contractAddress.address);
      expect(await transferLimiter.hasRole(CONTRACT_ROLE, contractAddress.address)).to.be.true;
    });

    it("Should revert when authorizing zero address as contract", async function () {
      const { transferLimiter } = await loadFixture(deployTransferLimiterFixture);
      
      await expect(transferLimiter.authorizeContract(ethers.ZeroAddress))
        .to.be.revertedWithCustomError(transferLimiter, "ZeroAddress");
    });
  });

  describe("Default Settings", function () {
    it("Should set default max transfer amount", async function () {
      const { transferLimiter, tokenAddress } = await loadFixture(deployTransferLimiterFixture);
      
      const amount = ethers.parseEther("100");
      await transferLimiter.setDefaultMaxTransferAmount(tokenAddress.address, amount);
      
      expect(await transferLimiter.defaultMaxTransferAmount(tokenAddress.address)).to.equal(amount);
    });

    it("Should revert when setting default max transfer amount to zero", async function () {
      const { transferLimiter, tokenAddress } = await loadFixture(deployTransferLimiterFixture);
      
      await expect(transferLimiter.setDefaultMaxTransferAmount(tokenAddress.address, 0))
        .to.be.revertedWithCustomError(transferLimiter, "AmountTooSmall");
    });

    it("Should set default cooldown", async function () {
      const { transferLimiter, tokenAddress } = await loadFixture(deployTransferLimiterFixture);
      
      const cooldown = 3600; // 1 hour in seconds
      await transferLimiter.setDefaultCooldown(tokenAddress.address, cooldown);
      
      expect(await transferLimiter.defaultTransferCooldown(tokenAddress.address)).to.equal(cooldown);
    });

    it("Should set default period limit", async function () {
      const { transferLimiter, tokenAddress } = await loadFixture(deployTransferLimiterFixture);
      
      const amount = ethers.parseEther("1000");
      const periodSeconds = 86400; // 24 hours in seconds
      await transferLimiter.setDefaultPeriodLimit(tokenAddress.address, amount, periodSeconds);
      
      expect(await transferLimiter.defaultPeriodLimit(tokenAddress.address)).to.equal(amount);
      expect(await transferLimiter.defaultPeriodDuration(tokenAddress.address)).to.equal(periodSeconds);
    });
  });

  describe("User-specific Settings", function () {
    it("Should set user max transfer amount", async function () {
      const { transferLimiter, tokenAddress, user1, limitManager } = await loadFixture(deployTransferLimiterFixture);
      
      // Add limit manager role
      await transferLimiter.addLimitManager(limitManager.address);
      
      const amount = ethers.parseEther("50");
      await transferLimiter.connect(limitManager).setUserMaxTransferAmount(tokenAddress.address, user1.address, amount);
      
      expect(await transferLimiter.userMaxTransferAmount(tokenAddress.address, user1.address)).to.equal(amount);
      expect(await transferLimiter.hasCustomLimits(tokenAddress.address, user1.address)).to.be.true;
    });

    it("Should set user cooldown", async function () {
      const { transferLimiter, tokenAddress, user1, limitManager } = await loadFixture(deployTransferLimiterFixture);
      
      await transferLimiter.addLimitManager(limitManager.address);
      
      const cooldown = 1800; // 30 minutes in seconds
      await transferLimiter.connect(limitManager).setUserCooldown(tokenAddress.address, user1.address, cooldown);
      
      expect(await transferLimiter.userTransferCooldown(tokenAddress.address, user1.address)).to.equal(cooldown);
      expect(await transferLimiter.hasCustomCooldown(tokenAddress.address, user1.address)).to.be.true;
    });

    it("Should set user period limit", async function () {
      const { transferLimiter, tokenAddress, user1, limitManager } = await loadFixture(deployTransferLimiterFixture);
      
      await transferLimiter.addLimitManager(limitManager.address);
      
      const amount = ethers.parseEther("500");
      const periodSeconds = 43200; // 12 hours in seconds
      await transferLimiter.connect(limitManager).setUserPeriodLimit(
        tokenAddress.address, user1.address, amount, periodSeconds
      );
      
      expect(await transferLimiter.userPeriodLimit(tokenAddress.address, user1.address)).to.equal(amount);
      expect(await transferLimiter.userPeriodDuration(tokenAddress.address, user1.address)).to.equal(periodSeconds);
      expect(await transferLimiter.hasCustomPeriodLimit(tokenAddress.address, user1.address)).to.be.true;
    });

    it("Should reset user to default settings", async function () {
      const { transferLimiter, tokenAddress, user1, limitManager } = await loadFixture(deployTransferLimiterFixture);
      
      await transferLimiter.addLimitManager(limitManager.address);
      
      // Set custom user settings
      await transferLimiter.connect(limitManager).setUserMaxTransferAmount(
        tokenAddress.address, user1.address, ethers.parseEther("50")
      );
      await transferLimiter.connect(limitManager).setUserCooldown(
        tokenAddress.address, user1.address, 1800
      );
      await transferLimiter.connect(limitManager).setUserPeriodLimit(
        tokenAddress.address, user1.address, ethers.parseEther("500"), 43200
      );
      
      // Reset to defaults
      await transferLimiter.connect(limitManager).resetUserToDefault(tokenAddress.address, user1.address);
      
      expect(await transferLimiter.hasCustomLimits(tokenAddress.address, user1.address)).to.be.false;
      expect(await transferLimiter.hasCustomCooldown(tokenAddress.address, user1.address)).to.be.false;
      expect(await transferLimiter.hasCustomPeriodLimit(tokenAddress.address, user1.address)).to.be.false;
    });
  });

  describe("Exemptions", function () {
    it("Should set exemption status", async function () {
      const { transferLimiter, tokenAddress, user1 } = await loadFixture(deployTransferLimiterFixture);
      
      // Set user as exempt
      await transferLimiter.setExemption(tokenAddress.address, user1.address, true);
      expect(await transferLimiter.exemptFromLimits(tokenAddress.address, user1.address)).to.be.true;
      
      // Remove exemption
      await transferLimiter.setExemption(tokenAddress.address, user1.address, false);
      expect(await transferLimiter.exemptFromLimits(tokenAddress.address, user1.address)).to.be.false;
    });

    it("Should batch set exemptions", async function () {
      const { transferLimiter, tokenAddress, user1, user2 } = await loadFixture(deployTransferLimiterFixture);
      
      const accounts = [user1.address, user2.address];
      await transferLimiter.batchSetExemptions(tokenAddress.address, accounts, true);
      
      expect(await transferLimiter.exemptFromLimits(tokenAddress.address, user1.address)).to.be.true;
      expect(await transferLimiter.exemptFromLimits(tokenAddress.address, user2.address)).to.be.true;
    });
  });

  describe("Batch Operations", function () {
    it("Should batch set user limits", async function () {
      const { transferLimiter, tokenAddress, limitManager, user1, user2 } = await loadFixture(deployTransferLimiterFixture);
      
      await transferLimiter.addLimitManager(limitManager.address);
      
      const users = [user1.address, user2.address];
      const amounts = [ethers.parseEther("50"), ethers.parseEther("75")];
      const cooldowns = [1800, 3600];
      const periodLimits = [ethers.parseEther("500"), ethers.parseEther("750")];
      const periodDurations = [43200, 86400];
      
      await transferLimiter.connect(limitManager).batchSetUserLimits(
        tokenAddress.address, users, amounts, cooldowns, periodLimits, periodDurations
      );
      
      expect(await transferLimiter.userMaxTransferAmount(tokenAddress.address, user1.address)).to.equal(amounts[0]);
      expect(await transferLimiter.userMaxTransferAmount(tokenAddress.address, user2.address)).to.equal(amounts[1]);
      expect(await transferLimiter.userTransferCooldown(tokenAddress.address, user1.address)).to.equal(cooldowns[0]);
      expect(await transferLimiter.userTransferCooldown(tokenAddress.address, user2.address)).to.equal(cooldowns[1]);
    });

    it("Should revert on array length mismatch", async function () {
      const { transferLimiter, tokenAddress, limitManager, user1, user2 } = await loadFixture(deployTransferLimiterFixture);
      
      await transferLimiter.addLimitManager(limitManager.address);
      
      const users = [user1.address, user2.address];
      const amounts = [ethers.parseEther("50")]; // Only one element
      const cooldowns = [1800, 3600];
      const periodLimits = [ethers.parseEther("500"), ethers.parseEther("750")];
      const periodDurations = [43200, 86400];
      
      await expect(transferLimiter.connect(limitManager).batchSetUserLimits(
        tokenAddress.address, users, amounts, cooldowns, periodLimits, periodDurations
      )).to.be.revertedWithCustomError(transferLimiter, "ArrayLengthMismatch");
    });
  });

  describe("Transfer Limit Checking", function () {
    it("Should pass transfer limit check for exempt users", async function () {
      const { transferLimiter, tokenAddress, user1, contractAddress } = await loadFixture(deployTransferLimiterFixture);
      
      // Set default limit
      await transferLimiter.setDefaultMaxTransferAmount(tokenAddress.address, ethers.parseEther("100"));
      
      // Set user as exempt
      await transferLimiter.setExemption(tokenAddress.address, user1.address, true);
      
      // Check large transfer amount (above the limit)
      const result = await transferLimiter.checkTransferLimit(
        tokenAddress.address, user1.address, ethers.parseEther("200")
      );
      
      expect(result).to.be.true;
    });
    
    it("Should fail transfer limit check when exceeding max amount", async function () {
      const { transferLimiter, tokenAddress, user1 } = await loadFixture(deployTransferLimiterFixture);
      
      // Set default limit
      await transferLimiter.setDefaultMaxTransferAmount(tokenAddress.address, ethers.parseEther("100"));
      
      // Check transfer above limit
      const result = await transferLimiter.checkTransferLimit(
        tokenAddress.address, user1.address, ethers.parseEther("150")
      );
      
      expect(result).to.be.false;
    });
    
    it("Should pass transfer limit check when under max amount", async function () {
      const { transferLimiter, tokenAddress, user1 } = await loadFixture(deployTransferLimiterFixture);
      
      // Set default limit
      await transferLimiter.setDefaultMaxTransferAmount(tokenAddress.address, ethers.parseEther("100"));
      
      // Check transfer below limit
      const result = await transferLimiter.checkTransferLimit(
        tokenAddress.address, user1.address, ethers.parseEther("50")
      );
      
      expect(result).to.be.true;
    });
  });

  describe("Cooldown Enforcement", function () {
    it("Should enforce cooldown period", async function () {
      const { transferLimiter, tokenAddress, user1, contractAddress } = await loadFixture(deployTransferLimiterFixture);
      
      // Set default cooldown to 1 hour
      await transferLimiter.setDefaultCooldown(tokenAddress.address, 3600);
      
      // First transfer should pass
      await transferLimiter.enforceCooldown(tokenAddress.address, user1.address);
      
      // Immediate second transfer should fail
      await expect(transferLimiter.enforceCooldown(tokenAddress.address, user1.address))
        .to.be.revertedWithCustomError(transferLimiter, "CooldownNotElapsed");
    });
    
    it("Should allow transfer after cooldown period", async function () {
      const { transferLimiter, tokenAddress, user1 } = await loadFixture(deployTransferLimiterFixture);
      
      // Set default cooldown to 1 hour
      await transferLimiter.setDefaultCooldown(tokenAddress.address, 3600);
      
      // First transfer
      await transferLimiter.enforceCooldown(tokenAddress.address, user1.address);
      
      // Advance time by more than the cooldown
      await time.increase(4000);
      
      // Second transfer should succeed
      await expect(transferLimiter.enforceCooldown(tokenAddress.address, user1.address))
        .to.not.be.reverted;
    });
    
    it("Should skip cooldown for exempt users", async function () {
      const { transferLimiter, tokenAddress, user1 } = await loadFixture(deployTransferLimiterFixture);
      
      // Set default cooldown to 1 hour
      await transferLimiter.setDefaultCooldown(tokenAddress.address, 3600);
      
      // Set user as exempt
      await transferLimiter.setExemption(tokenAddress.address, user1.address, true);
      
      // Multiple transfers should pass without cooldown
      await transferLimiter.enforceCooldown(tokenAddress.address, user1.address);
      await transferLimiter.enforceCooldown(tokenAddress.address, user1.address);
      await transferLimiter.enforceCooldown(tokenAddress.address, user1.address);
    });
  });

  describe("Period Limit Tracking", function () {
    it("Should track transfers within a period", async function () {
      const { transferLimiter, tokenAddress, user1, contractAddress } = await loadFixture(deployTransferLimiterFixture);
      
      // Authorize contract to call recordTransfer
      await transferLimiter.authorizeContract(contractAddress.address);
      
      // Set period limit to 100 tokens over 24 hours
      await transferLimiter.setDefaultPeriodLimit(tokenAddress.address, ethers.parseEther("100"), 86400);
      
      // Record transfer of 30 tokens
      await transferLimiter.connect(contractAddress).recordTransfer(
        tokenAddress.address, user1.address, ethers.parseEther("30")
      );
      
      // Check period total
      expect(await transferLimiter.periodTotalTransferred(tokenAddress.address, user1.address))
        .to.equal(ethers.parseEther("30"));
      
      // Record another transfer of 40 tokens
      await transferLimiter.connect(contractAddress).recordTransfer(
        tokenAddress.address, user1.address, ethers.parseEther("40")
      );
      
      // Check updated period total
      expect(await transferLimiter.periodTotalTransferred(tokenAddress.address, user1.address))
        .to.equal(ethers.parseEther("70"));
    });
    
    it("Should reject transfer that exceeds period limit", async function () {
      const { transferLimiter, tokenAddress, user1, contractAddress } = await loadFixture(deployTransferLimiterFixture);
      
      // Authorize contract to call recordTransfer
      await transferLimiter.authorizeContract(contractAddress.address);
      
      // Set period limit to 100 tokens over 24 hours
      await transferLimiter.setDefaultPeriodLimit(tokenAddress.address, ethers.parseEther("100"), 86400);
      
      // Record transfer of 70 tokens
      await transferLimiter.connect(contractAddress).recordTransfer(
        tokenAddress.address, user1.address, ethers.parseEther("70")
      );
      
      // Trying to transfer another 40 tokens should fail (exceeds 100 limit)
      await expect(transferLimiter.connect(contractAddress).recordTransfer(
        tokenAddress.address, user1.address, ethers.parseEther("40")
      )).to.be.revertedWithCustomError(transferLimiter, "ExceedsPeriodLimit");
    });
    
    it("Should reset period counter after period expiration", async function () {
      const { transferLimiter, tokenAddress, user1, contractAddress } = await loadFixture(deployTransferLimiterFixture);
      
      // Authorize contract to call recordTransfer
      await transferLimiter.authorizeContract(contractAddress.address);
      
      // Set period limit to 100 tokens over 1 hour
      await transferLimiter.setDefaultPeriodLimit(tokenAddress.address, ethers.parseEther("100"), 3600);
      
      // Record transfer of 70 tokens
      await transferLimiter.connect(contractAddress).recordTransfer(
        tokenAddress.address, user1.address, ethers.parseEther("70")
      );
      
      // Advance time beyond the period
      await time.increase(4000);
      
      // Record another transfer of 70 tokens (should succeed because period reset)
      await transferLimiter.connect(contractAddress).recordTransfer(
        tokenAddress.address, user1.address, ethers.parseEther("70")
      );
      
      // Period total should be 70 (reset from previous period)
      expect(await transferLimiter.periodTotalTransferred(tokenAddress.address, user1.address))
        .to.equal(ethers.parseEther("70"));
    });
    
    it("Should initialize period reset time on first transfer", async function () {
      const { transferLimiter, tokenAddress, user1, contractAddress } = await loadFixture(deployTransferLimiterFixture);
      
      // Authorize contract to call recordTransfer
      await transferLimiter.authorizeContract(contractAddress.address);
      
      // Set period limit to 100 tokens over 1 hour
      await transferLimiter.setDefaultPeriodLimit(tokenAddress.address, ethers.parseEther("100"), 3600);
      
      // Before transfer, reset time should be 0
      expect(await transferLimiter.periodResetTime(tokenAddress.address, user1.address)).to.equal(0);
      
      // Record transfer
      await transferLimiter.connect(contractAddress).recordTransfer(
        tokenAddress.address, user1.address, ethers.parseEther("30")
      );
      
      // After transfer, reset time should be set
      const resetTime = await transferLimiter.periodResetTime(tokenAddress.address, user1.address);
      expect(resetTime).to.be.gt(0);
    });
    
    it("Should manually reset user period", async function () {
      const { transferLimiter, tokenAddress, user1, limitManager, contractAddress } = await loadFixture(deployTransferLimiterFixture);
      
      // Add limit manager role
      await transferLimiter.addLimitManager(limitManager.address);
      
      // Authorize contract to call recordTransfer
      await transferLimiter.authorizeContract(contractAddress.address);
      
      // Set period limit
      await transferLimiter.setDefaultPeriodLimit(tokenAddress.address, ethers.parseEther("100"), 3600);
      
      // Record transfer
      await transferLimiter.connect(contractAddress).recordTransfer(
        tokenAddress.address, user1.address, ethers.parseEther("70")
      );
      
      // Manually reset period
      await transferLimiter.connect(limitManager).resetUserPeriod(tokenAddress.address, user1.address);
      
      // Period total should be reset to 0
      expect(await transferLimiter.periodTotalTransferred(tokenAddress.address, user1.address))
        .to.equal(0);
      
      // Check that period reset time is updated
      const resetTime = await transferLimiter.periodResetTime(tokenAddress.address, user1.address);
      expect(resetTime).to.be.gt(0);
    });
  });

  describe("Getter Functions", function () {
    it("Should get effective max transfer amount", async function () {
      const { transferLimiter, tokenAddress, user1, limitManager } = await loadFixture(deployTransferLimiterFixture);
      
      // Set default max transfer amount
      await transferLimiter.setDefaultMaxTransferAmount(tokenAddress.address, ethers.parseEther("100"));
      
      // Check effective amount for user with default settings
      expect(await transferLimiter.getEffectiveMaxTransferAmount(tokenAddress.address, user1.address))
        .to.equal(ethers.parseEther("100"));
      
      // Add limit manager and set custom limit for user
      await transferLimiter.addLimitManager(limitManager.address);
      await transferLimiter.connect(limitManager).setUserMaxTransferAmount(
        tokenAddress.address, user1.address, ethers.parseEther("50")
      );
      
      // Check effective amount with custom settings
      expect(await transferLimiter.getEffectiveMaxTransferAmount(tokenAddress.address, user1.address))
        .to.equal(ethers.parseEther("50"));
      
      // Set user as exempt
      await transferLimiter.setExemption(tokenAddress.address, user1.address, true);
      
      // Exempt users should have no limit
      expect(await transferLimiter.getEffectiveMaxTransferAmount(tokenAddress.address, user1.address))
        .to.equal(ethers.MaxUint256);
    });
    
    it("Should get next valid transfer time", async function () {
      const { transferLimiter, tokenAddress, user1 } = await loadFixture(deployTransferLimiterFixture);
      
      // Set default cooldown to 1 hour
      await transferLimiter.setDefaultCooldown(tokenAddress.address, 3600);
      
      // Before any transfer, next valid time should be 0
      expect(await transferLimiter.getNextValidTransferTime(tokenAddress.address, user1.address))
        .to.equal(0);
      
      // Do first transfer
      await transferLimiter.enforceCooldown(tokenAddress.address, user1.address);
      
      // After transfer, next valid time should be in the future
      const nextValidTime = await transferLimiter.getNextValidTransferTime(tokenAddress.address, user1.address);
      expect(nextValidTime).to.be.gt(0);
      
      // Advance time beyond cooldown
      await time.increase(4000);
      
      // Next valid time should be 0 again (can transfer now)
      expect(await transferLimiter.getNextValidTransferTime(tokenAddress.address, user1.address))
        .to.equal(0);
    });
  });
});