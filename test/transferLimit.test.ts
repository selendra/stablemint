import { ethers } from "hardhat";
import { expect } from "chai";
import { time } from "@nomicfoundation/hardhat-network-helpers";
import { 
  TransferLimiter, 
  MockERC20 
} from "../typechain-types";
import { 
  SignerWithAddress 
} from "@nomicfoundation/hardhat-ethers/signers";
import { 
  Contract, 
  ContractTransactionResponse,
  MaxUint256
} from "ethers";

describe("TransferLimiter", function () {
  let transferLimiter: TransferLimiter;
  let mockToken: MockERC20;
  let owner: SignerWithAddress, 
      admin: SignerWithAddress, 
      limitManager: SignerWithAddress, 
      user1: SignerWithAddress, 
      user2: SignerWithAddress, 
      user3: SignerWithAddress, 
      contract1: SignerWithAddress;
  
  // Constants for roles
  const ADMIN_ROLE = ethers.keccak256(ethers.toUtf8Bytes("ADMIN_ROLE"));
  const CONTRACT_ROLE = ethers.keccak256(ethers.toUtf8Bytes("CONTRACT_ROLE"));
  const LIMIT_MANAGER_ROLE = ethers.keccak256(ethers.toUtf8Bytes("LIMIT_MANAGER_ROLE"));
  
  // Test values
  const DEFAULT_MAX_AMOUNT = ethers.parseEther("100");
  const DEFAULT_COOLDOWN = 3600; // 1 hour in seconds
  const USER_MAX_AMOUNT = ethers.parseEther("50");
  const USER_COOLDOWN = 1800; // 30 minutes

  beforeEach(async function () {
    // Get signers
    [owner, admin, limitManager, user1, user2, user3, contract1] = await ethers.getSigners();
    
    // Deploy mock ERC20 token
    const MockToken = await ethers.getContractFactory("MockERC20");
    mockToken = await MockToken.deploy("Mock Token", "MTK", 18) as MockERC20;
    await mockToken.waitForDeployment();
    
    // Deploy TransferLimiter contract
    const TransferLimiter = await ethers.getContractFactory("TransferLimiter");
    transferLimiter = await TransferLimiter.deploy() as TransferLimiter;
    await transferLimiter.waitForDeployment();
    
    // Setup roles
    await transferLimiter.grantRole(ADMIN_ROLE, admin.address);
    await transferLimiter.grantRole(LIMIT_MANAGER_ROLE, limitManager.address);
  });

  describe("Deployment", function () {
    it("should set the correct roles for the deployer", async function () {
      const DEFAULT_ADMIN_ROLE = "0x0000000000000000000000000000000000000000000000000000000000000000";
      
      expect(await transferLimiter.hasRole(DEFAULT_ADMIN_ROLE, owner.address)).to.be.true;
      expect(await transferLimiter.hasRole(ADMIN_ROLE, owner.address)).to.be.true;
      expect(await transferLimiter.hasRole(LIMIT_MANAGER_ROLE, owner.address)).to.be.true;
    });
  });

  describe("Access Control", function () {
    it("should allow admin to add limit manager", async function () {
      await transferLimiter.connect(admin).addLimitManager(user1.address);
      expect(await transferLimiter.hasRole(LIMIT_MANAGER_ROLE, user1.address)).to.be.true;
    });

    it("should allow admin to remove limit manager", async function () {
      await transferLimiter.connect(admin).addLimitManager(user1.address);
      await transferLimiter.connect(admin).removeLimitManager(user1.address);
      expect(await transferLimiter.hasRole(LIMIT_MANAGER_ROLE, user1.address)).to.be.false;
    });

    it("should allow admin to authorize contract", async function () {
      await transferLimiter.connect(admin).authorizeContract(contract1.address);
      expect(await transferLimiter.hasRole(CONTRACT_ROLE, contract1.address)).to.be.true;
    });

    it("should revert when non-admin tries to add limit manager", async function () {
      await expect(
        transferLimiter.connect(user1).addLimitManager(user2.address)
      ).to.be.revertedWithCustomError(transferLimiter, "AccessControlUnauthorizedAccount");
    });

    it("should revert when authorizing zero address as contract", async function () {
      await expect(
        transferLimiter.connect(admin).authorizeContract(ethers.ZeroAddress)
      ).to.be.revertedWithCustomError(transferLimiter, "ZeroAddress");
    });
  });

  describe("Default Limits", function () {
    it("should allow admin to set default max transfer amount", async function () {
      await transferLimiter.connect(admin).setDefaultMaxTransferAmount(mockToken.target, DEFAULT_MAX_AMOUNT);
      expect(await transferLimiter.defaultMaxTransferAmount(mockToken.target)).to.equal(DEFAULT_MAX_AMOUNT);
    });

    it("should allow admin to set default cooldown", async function () {
      await transferLimiter.connect(admin).setDefaultCooldown(mockToken.target, DEFAULT_COOLDOWN);
      expect(await transferLimiter.defaultTransferCooldown(mockToken.target)).to.equal(DEFAULT_COOLDOWN);
    });

    it("should revert when setting default max amount to zero", async function () {
      await expect(
        transferLimiter.connect(admin).setDefaultMaxTransferAmount(mockToken.target, 0)
      ).to.be.revertedWithCustomError(transferLimiter, "AmountTooSmall");
    });

    it("should emit event when setting default max transfer amount", async function () {
      await expect(transferLimiter.connect(admin).setDefaultMaxTransferAmount(mockToken.target, DEFAULT_MAX_AMOUNT))
        .to.emit(transferLimiter, "DefaultMaxTransferUpdated")
        .withArgs(mockToken.target, DEFAULT_MAX_AMOUNT);
    });

    it("should emit event when setting default cooldown", async function () {
      await expect(transferLimiter.connect(admin).setDefaultCooldown(mockToken.target, DEFAULT_COOLDOWN))
        .to.emit(transferLimiter, "DefaultCooldownUpdated")
        .withArgs(mockToken.target, DEFAULT_COOLDOWN);
    });
  });

  describe("User-specific Limits", function () {
    it("should allow limit manager to set user max transfer amount", async function () {
      await transferLimiter.connect(limitManager).setUserMaxTransferAmount(
        mockToken.target, user1.address, USER_MAX_AMOUNT
      );
      
      expect(await transferLimiter.userMaxTransferAmount(mockToken.target, user1.address)).to.equal(USER_MAX_AMOUNT);
      expect(await transferLimiter.hasCustomLimits(mockToken.target, user1.address)).to.be.true;
    });

    it("should allow limit manager to set user cooldown", async function () {
      await transferLimiter.connect(limitManager).setUserCooldown(
        mockToken.target, user1.address, USER_COOLDOWN
      );
      
      expect(await transferLimiter.userTransferCooldown(mockToken.target, user1.address)).to.equal(USER_COOLDOWN);
      expect(await transferLimiter.hasCustomCooldown(mockToken.target, user1.address)).to.be.true;
    });

    it("should allow limit manager to reset user to default", async function () {
      // First set custom limits
      await transferLimiter.connect(limitManager).setUserMaxTransferAmount(
        mockToken.target, user1.address, USER_MAX_AMOUNT
      );
      await transferLimiter.connect(limitManager).setUserCooldown(
        mockToken.target, user1.address, USER_COOLDOWN
      );
      
      // Then reset to default
      await transferLimiter.connect(limitManager).resetUserToDefault(mockToken.target, user1.address);
      
      expect(await transferLimiter.hasCustomLimits(mockToken.target, user1.address)).to.be.false;
      expect(await transferLimiter.hasCustomCooldown(mockToken.target, user1.address)).to.be.false;
    });

    it("should revert when setting user max amount to zero", async function () {
      await expect(
        transferLimiter.connect(limitManager).setUserMaxTransferAmount(mockToken.target, user1.address, 0)
      ).to.be.revertedWithCustomError(transferLimiter, "AmountTooSmall");
    });
  });

  describe("Exemptions", function () {
    it("should allow admin to set exemption", async function () {
      await transferLimiter.connect(admin).setExemption(mockToken.target, user1.address, true);
      expect(await transferLimiter.exemptFromLimits(mockToken.target, user1.address)).to.be.true;
    });

    it("should emit event when setting exemption", async function () {
      await expect(transferLimiter.connect(admin).setExemption(mockToken.target, user1.address, true))
        .to.emit(transferLimiter, "ExemptionUpdated")
        .withArgs(mockToken.target, user1.address, true);
    });

    it("should allow admin to remove exemption", async function () {
      await transferLimiter.connect(admin).setExemption(mockToken.target, user1.address, true);
      await transferLimiter.connect(admin).setExemption(mockToken.target, user1.address, false);
      expect(await transferLimiter.exemptFromLimits(mockToken.target, user1.address)).to.be.false;
    });
  });

  describe("Transfer Limit Enforcement", function () {
    beforeEach(async function () {
      // Set up default limits
      await transferLimiter.connect(admin).setDefaultMaxTransferAmount(mockToken.target, DEFAULT_MAX_AMOUNT);
      await transferLimiter.connect(admin).setDefaultCooldown(mockToken.target, DEFAULT_COOLDOWN);
      
      // Set up user-specific limits for user1
      await transferLimiter.connect(limitManager).setUserMaxTransferAmount(
        mockToken.target, user1.address, USER_MAX_AMOUNT
      );
    });

    it("should allow transfer within default limits", async function () {
      const amountToTransfer = DEFAULT_MAX_AMOUNT - 1n;
      expect(
        await transferLimiter.checkTransferLimit(mockToken.target, user2.address, amountToTransfer)
      ).to.be.true;
    });

    it("should allow transfer within user-specific limits", async function () {
      const amountToTransfer = USER_MAX_AMOUNT - 1n;
      expect(
        await transferLimiter.checkTransferLimit(mockToken.target, user1.address, amountToTransfer)
      ).to.be.true;
    });

    it("should not allow transfer exceeding user-specific limits", async function () {
      const amountToTransfer = USER_MAX_AMOUNT + 1n;
      expect(
        await transferLimiter.checkTransferLimit(mockToken.target, user1.address, amountToTransfer)
      ).to.be.false;
    });

    it("should allow any transfer amount for exempt users", async function () {
      const largeAmount = DEFAULT_MAX_AMOUNT * 10n;
      await transferLimiter.connect(admin).setExemption(mockToken.target, user3.address, true);
      expect(
        await transferLimiter.checkTransferLimit(mockToken.target, user3.address, largeAmount)
      ).to.be.true;
    });
  });

  describe("Cooldown Enforcement", function () {
    beforeEach(async function () {
      // Set up default cooldown
      await transferLimiter.connect(admin).setDefaultCooldown(mockToken.target, DEFAULT_COOLDOWN);
      
      // Set up user-specific cooldown for user1
      await transferLimiter.connect(limitManager).setUserCooldown(
        mockToken.target, user1.address, USER_COOLDOWN
      );
    });

    it("should allow first transfer with default cooldown", async function () {
      expect(await transferLimiter.connect(contract1).enforceCooldown(mockToken.target, user2.address))
        .to.be.true;
    });

    it("should block second transfer before default cooldown expires", async function () {
      // First transfer
      await transferLimiter.connect(contract1).enforceCooldown(mockToken.target, user2.address);
      
      // Second transfer too soon
      await expect(
        transferLimiter.connect(contract1).enforceCooldown(mockToken.target, user2.address)
      ).to.be.revertedWithCustomError(transferLimiter, "CooldownNotElapsed");
    });

    it("should allow second transfer after default cooldown expires", async function () {
      // First transfer
      await transferLimiter.connect(contract1).enforceCooldown(mockToken.target, user2.address);
      
      // Increase time to expire cooldown
      await time.increase(DEFAULT_COOLDOWN + 1);
      
      // Second transfer should succeed
      expect(await transferLimiter.connect(contract1).enforceCooldown(mockToken.target, user2.address))
        .to.be.true;
    });

    it("should block second transfer before user-specific cooldown expires", async function () {
      // First transfer
      await transferLimiter.connect(contract1).enforceCooldown(mockToken.target, user1.address);
      
      // Second transfer too soon
      await expect(
        transferLimiter.connect(contract1).enforceCooldown(mockToken.target, user1.address)
      ).to.be.revertedWithCustomError(transferLimiter, "CooldownNotElapsed");
    });

    it("should allow transfer anytime for exempt users", async function () {
      await transferLimiter.connect(admin).setExemption(mockToken.target, user3.address, true);
      
      // First transfer
      await transferLimiter.connect(contract1).enforceCooldown(mockToken.target, user3.address);
      
      // Second transfer immediately should succeed
      expect(await transferLimiter.connect(contract1).enforceCooldown(mockToken.target, user3.address))
        .to.be.true;
    });
  });

  describe("Batch Operations", function () {
    it("should set batch user limits correctly", async function () {
      const users = [user1.address, user2.address, user3.address];
      const amounts = [ethers.parseEther("10"), ethers.parseEther("20"), ethers.parseEther("30")];
      const cooldowns = [600, 1200, 1800];
      
      await transferLimiter.connect(limitManager).batchSetUserLimits(
        mockToken.target, users, amounts, cooldowns
      );
      
      expect(await transferLimiter.userMaxTransferAmount(mockToken.target, user1.address)).to.equal(amounts[0]);
      expect(await transferLimiter.userMaxTransferAmount(mockToken.target, user2.address)).to.equal(amounts[1]);
      expect(await transferLimiter.userMaxTransferAmount(mockToken.target, user3.address)).to.equal(amounts[2]);
      
      expect(await transferLimiter.userTransferCooldown(mockToken.target, user1.address)).to.equal(cooldowns[0]);
      expect(await transferLimiter.userTransferCooldown(mockToken.target, user2.address)).to.equal(cooldowns[1]);
      expect(await transferLimiter.userTransferCooldown(mockToken.target, user3.address)).to.equal(cooldowns[2]);
    });

    it("should set batch exemptions correctly", async function () {
      const accounts = [user1.address, user2.address, user3.address];
      const status = true;
      
      await transferLimiter.connect(admin).batchSetExemptions(
        mockToken.target, accounts, status
      );
      
      expect(await transferLimiter.exemptFromLimits(mockToken.target, user1.address)).to.equal(status);
      expect(await transferLimiter.exemptFromLimits(mockToken.target, user2.address)).to.equal(status);
      expect(await transferLimiter.exemptFromLimits(mockToken.target, user3.address)).to.equal(status);
    });

    it("should revert batch user limits on array length mismatch", async function () {
      const users = [user1.address, user2.address];
      const amounts = [ethers.parseEther("10")]; // Shorter array
      const cooldowns = [600, 1200];
      
      await expect(
        transferLimiter.connect(limitManager).batchSetUserLimits(mockToken.target, users, amounts, cooldowns)
      ).to.be.revertedWith("Array length mismatch");
    });
  });

  describe("Helper Functions", function () {
    beforeEach(async function () {
      // Set up default limits
      await transferLimiter.connect(admin).setDefaultMaxTransferAmount(mockToken.target, DEFAULT_MAX_AMOUNT);
      await transferLimiter.connect(admin).setDefaultCooldown(mockToken.target, DEFAULT_COOLDOWN);
      
      // Set up user-specific limits for user1
      await transferLimiter.connect(limitManager).setUserMaxTransferAmount(
        mockToken.target, user1.address, USER_MAX_AMOUNT
      );
      await transferLimiter.connect(limitManager).setUserCooldown(
        mockToken.target, user1.address, USER_COOLDOWN
      );
      
      // Set up exemption for user3
      await transferLimiter.connect(admin).setExemption(mockToken.target, user3.address, true);
    });

    it("should return correct effective max transfer amount", async function () {
      expect(await transferLimiter.getEffectiveMaxTransferAmount(mockToken.target, user1.address))
        .to.equal(USER_MAX_AMOUNT);
      expect(await transferLimiter.getEffectiveMaxTransferAmount(mockToken.target, user2.address))
        .to.equal(DEFAULT_MAX_AMOUNT);
      expect(await transferLimiter.getEffectiveMaxTransferAmount(mockToken.target, user3.address))
        .to.equal(MaxUint256);
    });

    it("should return correct effective cooldown period", async function () {
      expect(await transferLimiter.getEffectiveCooldownPeriod(mockToken.target, user1.address))
        .to.equal(USER_COOLDOWN);
      expect(await transferLimiter.getEffectiveCooldownPeriod(mockToken.target, user2.address))
        .to.equal(DEFAULT_COOLDOWN);
      expect(await transferLimiter.getEffectiveCooldownPeriod(mockToken.target, user3.address))
        .to.equal(0);
    });

    // it("should return correct next valid transfer time", async function () {
    //   // First transfer for user1
    //   await transferLimiter.connect(contract1).enforceCooldown(mockToken.target, user1.address);
      
    //   const expectedNextTime = await time.latest() + BigInt(USER_COOLDOWN);
    //   const actualNextTime = await transferLimiter.getNextValidTransferTime(mockToken.target, user1.address);
      
    //   // Allow for small timestamp differences
    //   expect(actualNextTime).to.be.closeTo(expectedNextTime, 2);
      
    //   // Exempt user should have 0 next valid time
    //   expect(await transferLimiter.getNextValidTransferTime(mockToken.target, user3.address))
    //     .to.equal(0);
    // });
  });
});