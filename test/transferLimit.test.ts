import { expect } from "chai";
import { ethers } from "hardhat";
import { time } from "@nomicfoundation/hardhat-network-helpers";
import { HardhatEthersSigner } from "@nomicfoundation/hardhat-ethers/signers";
import { TransferLimiter } from "../typechain-types";

describe("TransferLimiter", function () {
  let transferLimiter: TransferLimiter;
  let admin: HardhatEthersSigner, limitManager: HardhatEthersSigner, 
      user1: HardhatEthersSigner, user2: HardhatEthersSigner, contractMock: HardhatEthersSigner;
  let tokenAddress: string;
  
  const oneHour = 3600n;
  const oneDay = 86400n;
  
  // Test limit configuration
  const defaultLimits = {
    maxTransferAmount: ethers.parseUnits("10000", 18),
    cooldownPeriod: 600n, // 10 minutes
    periodLimit: ethers.parseUnits("50000", 18),
    periodDuration: oneDay
  };
  
  beforeEach(async function () {
    [admin, limitManager, user1, user2, contractMock] = await ethers.getSigners();
    
    // Generate random token address
    const randomBytes = ethers.randomBytes(20);
    tokenAddress = ethers.getAddress(ethers.hexlify(randomBytes));
    
    const TransferLimiter = await ethers.getContractFactory("TransferLimiter");
    transferLimiter = await TransferLimiter.deploy() as TransferLimiter;
    await transferLimiter.waitForDeployment();
    
    // Setup default limits for testing
    await transferLimiter.setAllDefaultLimits(tokenAddress, defaultLimits);
    
    // Add limit manager role
    await transferLimiter.addLimitManager(limitManager.address);
    
    // Authorize contract for testing
    await transferLimiter.authorizeContract(contractMock.address);
  });
  
  describe("Initialization", function () {
    it("should set deployer as admin and limit manager", async function () {
      const DEFAULT_ADMIN_ROLE = await transferLimiter.DEFAULT_ADMIN_ROLE();
      const ADMIN_ROLE = await transferLimiter.ADMIN_ROLE();
      const LIMIT_MANAGER_ROLE = await transferLimiter.LIMIT_MANAGER_ROLE();
      
      expect(await transferLimiter.hasRole(DEFAULT_ADMIN_ROLE, admin.address)).to.be.true;
      expect(await transferLimiter.hasRole(ADMIN_ROLE, admin.address)).to.be.true;
      expect(await transferLimiter.hasRole(LIMIT_MANAGER_ROLE, admin.address)).to.be.true;
    });
    
    it("should correctly set default limits", async function () {
      const limits = await transferLimiter.defaultLimits(tokenAddress);
      expect(limits.maxTransferAmount).to.equal(defaultLimits.maxTransferAmount);
      expect(limits.cooldownPeriod).to.equal(defaultLimits.cooldownPeriod);
      expect(limits.periodLimit).to.equal(defaultLimits.periodLimit);
      expect(limits.periodDuration).to.equal(defaultLimits.periodDuration);
    });
  });
  
  describe("Role Management", function () {
    it("should allow admin to add a limit manager", async function () {
      const LIMIT_MANAGER_ROLE = await transferLimiter.LIMIT_MANAGER_ROLE();
      await transferLimiter.addLimitManager(user1.address);
      expect(await transferLimiter.hasRole(LIMIT_MANAGER_ROLE, user1.address)).to.be.true;
    });
    
    it("should allow admin to remove a limit manager", async function () {
      const LIMIT_MANAGER_ROLE = await transferLimiter.LIMIT_MANAGER_ROLE();
      await transferLimiter.addLimitManager(user1.address);
      await transferLimiter.removeLimitManager(user1.address);
      expect(await transferLimiter.hasRole(LIMIT_MANAGER_ROLE, user1.address)).to.be.false;
    });
  });
  
  describe("Transfer Limit Checking", function () {
    it("should allow transfers within single transfer limit", async function () {
      const amount = defaultLimits.maxTransferAmount / 2n;
      expect(await transferLimiter.checkTransferLimit(tokenAddress, user1.address, amount)).to.be.true;
    });
    
    it("should reject transfers exceeding single transfer limit", async function () {
      const amount = defaultLimits.maxTransferAmount + 1n;
      expect(await transferLimiter.checkTransferLimit(tokenAddress, user1.address, amount)).to.be.false;
    });
    
    it("should apply custom user limits when set", async function () {
      const customLimit = defaultLimits.maxTransferAmount / 2n;
      await transferLimiter.connect(limitManager).setUserMaxTransferAmount(tokenAddress, user1.address, customLimit);
      
      // Should pass with amount below custom limit
      expect(await transferLimiter.checkTransferLimit(tokenAddress, user1.address, customLimit - 1n)).to.be.true;
      
      // Should fail with amount above custom limit
      expect(await transferLimiter.checkTransferLimit(tokenAddress, user1.address, customLimit + 1n)).to.be.false;
    });
  });
  
  describe("Cooldown Enforcement", function () {
    it("should allow first transfer without cooldown", async function () {
      await expect(
        transferLimiter.connect(contractMock).enforceCooldown(tokenAddress, user1.address)
      ).not.to.be.reverted;
    });
    
    it("should enforce cooldown between transfers", async function () {
      // First transfer goes through
      await transferLimiter.connect(contractMock).enforceCooldown(tokenAddress, user1.address);
      
      // Second transfer should be rejected due to cooldown
      await expect(
        transferLimiter.connect(contractMock).enforceCooldown(tokenAddress, user1.address)
      ).to.be.revertedWithCustomError(transferLimiter, "CooldownNotElapsed");
      
      // Advance time past cooldown period
      await time.increase(Number(defaultLimits.cooldownPeriod) + 1);
      
      // Now transfer should succeed
      await expect(
        transferLimiter.connect(contractMock).enforceCooldown(tokenAddress, user1.address)
      ).not.to.be.reverted;
    });
  });
  
  describe("Period Limits", function () {
    beforeEach(async function () {
      // Ensure cooldown check passes for testing period limits
      await transferLimiter.setDefaultCooldown(tokenAddress, 0n);
    });
    
    it("should track period totals correctly", async function () {
      const amount = ethers.parseUnits("1000", 18);
      
      // Record first transfer
      await transferLimiter.connect(contractMock).recordTransfer(tokenAddress, user1.address, amount);
      
      // Get remaining allowance
      const [remaining, resetTime] = await transferLimiter.getRemainingPeriodAllowance(tokenAddress, user1.address);
      expect(remaining).to.equal(defaultLimits.periodLimit - amount);
      expect(resetTime).to.be.gt(0n); // Reset time should be set
      
      // Record second transfer
      await transferLimiter.connect(contractMock).recordTransfer(tokenAddress, user1.address, amount);
      
      // Check updated remaining allowance
      const [updatedRemaining] = await transferLimiter.getRemainingPeriodAllowance(tokenAddress, user1.address);
      expect(updatedRemaining).to.equal(defaultLimits.periodLimit - (amount * 2n));
    });
    
    it("should reject transfers exceeding period limit", async function () {
      // Record transfers up to the limit
      const amount = defaultLimits.periodLimit;
      await transferLimiter.connect(contractMock).recordTransfer(tokenAddress, user1.address, amount);
      
      // Next transfer should fail
      const smallAmount = ethers.parseUnits("1", 18);
      await expect(
        transferLimiter.connect(contractMock).recordTransfer(tokenAddress, user1.address, smallAmount)
      ).to.be.revertedWithCustomError(transferLimiter, "ExceedsPeriodLimit");
    });
    
    it("should reset period counter after period duration", async function () {
      // Record a transfer
      const amount = defaultLimits.periodLimit / 2n;
      await transferLimiter.connect(contractMock).recordTransfer(tokenAddress, user1.address, amount);
      
      // Advance time past period duration
      await time.increase(Number(defaultLimits.periodDuration) + 1);
      
      // Record another transfer - should now have a fresh period limit
      await expect(
        transferLimiter.connect(contractMock).recordTransfer(tokenAddress, user1.address, amount)
      ).not.to.be.reverted;
      
      // Check remaining is from a fresh period (full limit minus current transfer)
      const [remaining] = await transferLimiter.getRemainingPeriodAllowance(tokenAddress, user1.address);
      expect(remaining).to.equal(defaultLimits.periodLimit - amount);
    });
  });
  
  describe("Exemptions", function () {
    it("should exempt addresses from all limits", async function () {
      // Set exemption
      await transferLimiter.setExemption(tokenAddress, user1.address, true);
      
      // Should pass limit check even for large amount
      const largeAmount = defaultLimits.maxTransferAmount * 100n;
      expect(await transferLimiter.checkTransferLimit(tokenAddress, user1.address, largeAmount)).to.be.true;
      
      // Should pass cooldown check multiple times
      await transferLimiter.connect(contractMock).enforceCooldown(tokenAddress, user1.address);
      await transferLimiter.connect(contractMock).enforceCooldown(tokenAddress, user1.address);
      
      // Should pass period limit check for large amount
      await transferLimiter.connect(contractMock).recordTransfer(tokenAddress, user1.address, largeAmount);
    });
    
    it("should allow batch exemption setting", async function () {
      const addresses = [user1.address, user2.address];
      await transferLimiter.batchSetExemptions(tokenAddress, addresses, true);
      
      // Check all addresses are exempt
      for (const addr of addresses) {
        const transferState = await transferLimiter.transferState(tokenAddress, addr);
        expect(transferState.exempt).to.be.true;
      }
    });
  });
  
  describe("User-specific limits", function () {
    it("should allow setting all user limits at once", async function () {
      const userConfig = {
        maxTransferAmount: ethers.parseUnits("5000", 18),
        cooldownPeriod: 300n, // 5 minutes
        periodLimit: ethers.parseUnits("20000", 18),
        periodDuration: oneHour * 12n,
        hasCustomLimits: true,
        hasCustomCooldown: true,
        hasCustomPeriodLimit: true
      };
      
      await transferLimiter.connect(limitManager).setAllUserLimits(tokenAddress, user1.address, userConfig);
      
      // Verify the effective limits match the custom ones
      expect(await transferLimiter.getEffectiveMaxTransferAmount(tokenAddress, user1.address))
        .to.equal(userConfig.maxTransferAmount);
      expect(await transferLimiter.getEffectiveCooldownPeriod(tokenAddress, user1.address))
        .to.equal(userConfig.cooldownPeriod);
      expect(await transferLimiter.getEffectivePeriodLimit(tokenAddress, user1.address))
        .to.equal(userConfig.periodLimit);
      expect(await transferLimiter.getEffectivePeriodDuration(tokenAddress, user1.address))
        .to.equal(userConfig.periodDuration);
    });
    
    it("should reset user to default limits", async function () {
      // First set custom limits
      const userConfig = {
        maxTransferAmount: ethers.parseUnits("5000", 18),
        cooldownPeriod: 300n,
        periodLimit: ethers.parseUnits("20000", 18),
        periodDuration: oneHour * 12n,
        hasCustomLimits: true,
        hasCustomCooldown: true,
        hasCustomPeriodLimit: true
      };
      
      await transferLimiter.connect(limitManager).setAllUserLimits(tokenAddress, user1.address, userConfig);
      
      // Now reset to defaults
      await transferLimiter.connect(limitManager).resetUserToDefault(tokenAddress, user1.address);
      
      // Verify effective limits match defaults
      expect(await transferLimiter.getEffectiveMaxTransferAmount(tokenAddress, user1.address))
        .to.equal(defaultLimits.maxTransferAmount);
      expect(await transferLimiter.getEffectiveCooldownPeriod(tokenAddress, user1.address))
        .to.equal(defaultLimits.cooldownPeriod);
      expect(await transferLimiter.getEffectivePeriodLimit(tokenAddress, user1.address))
        .to.equal(defaultLimits.periodLimit);
      expect(await transferLimiter.getEffectivePeriodDuration(tokenAddress, user1.address))
        .to.equal(defaultLimits.periodDuration);
    });
  });
  
  describe("Events", function () {
    it("should emit LimitUpdated event when limits change", async function () {
      await expect(transferLimiter.setDefaultMaxTransferAmount(tokenAddress, ethers.parseUnits("20000", 18)))
        .to.emit(transferLimiter, "LimitUpdated");
    });
    
    it("should emit ExemptionUpdated event when exemption status changes", async function () {
      await expect(transferLimiter.setExemption(tokenAddress, user1.address, true))
        .to.emit(transferLimiter, "ExemptionUpdated")
        .withArgs(tokenAddress, user1.address, true);
    });
    
    it("should emit TransferRecorded event when transfer is recorded", async function () {
      const amount = ethers.parseUnits("1000", 18);
      await expect(transferLimiter.connect(contractMock).recordTransfer(tokenAddress, user1.address, amount))
        .to.emit(transferLimiter, "TransferRecorded")
        .withArgs(tokenAddress, user1.address, amount, amount);
    });
  });
});