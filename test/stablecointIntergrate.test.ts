import { expect } from "chai";
import { ethers } from "hardhat";
import { time } from "@nomicfoundation/hardhat-network-helpers";
import { HardhatEthersSigner } from "@nomicfoundation/hardhat-ethers/signers";
import { StableCoin, Whitelist, TransferLimiter } from "../typechain-types";

describe("StableCoin System Integration", function () {
  let stableCoin: StableCoin;
  let whitelist: Whitelist;
  let transferLimiter: TransferLimiter;
  let admin: HardhatEthersSigner, user1: HardhatEthersSigner, 
      user2: HardhatEthersSigner, user3: HardhatEthersSigner;
  
  const name = "Integrated Stable Coin";
  const symbol = "ISC";
  const initialSupply = 1000000n; // 1 million
  
  beforeEach(async function () {
    [admin, user1, user2, user3] = await ethers.getSigners();
    
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
    
    const stableCoinAddress = await stableCoin.getAddress();
    
    // Configure systems
    
    // 1. Set default limits in TransferLimiter
    const limitConfig = {
      maxTransferAmount: ethers.parseUnits("10000", 18),
      cooldownPeriod: 60n,  // 1 minute cooldown
      periodLimit: ethers.parseUnits("50000", 18),
      periodDuration: 86400n // 1 day period
    };
    await transferLimiter.setAllDefaultLimits(stableCoinAddress, limitConfig);
    
    // 2. Add StableCoin to authorized contracts in both helpers
    const CONTRACT_ROLE_WHITELIST = await whitelist.CONTRACT_ROLE();
    const CONTRACT_ROLE_LIMITER = await transferLimiter.CONTRACT_ROLE();
    
    if (!(await whitelist.hasRole(CONTRACT_ROLE_WHITELIST, stableCoinAddress))) {
      await whitelist.authorizeContract(stableCoinAddress);
    }
    
    if (!(await transferLimiter.hasRole(CONTRACT_ROLE_LIMITER, stableCoinAddress))) {
      await transferLimiter.authorizeContract(stableCoinAddress);
    }
    
    // 3. Whitelist test users
    await whitelist.batchSetWhitelisted([user1.address, user2.address], true);
    
    // 4. Exempt admin from transfer limits
    await transferLimiter.setExemption(stableCoinAddress, admin.address, true);
    
    // 5. Send some tokens to users for testing
    await stableCoin.transfer(user1.address, ethers.parseUnits("20000", 18));
  });
  
  describe("End-to-End Transfer Scenarios", function () {
    it("should handle a complete transfer cycle with all checks", async function () {
      // 1. Initial balances
      const transferAmount = ethers.parseUnits("5000", 18);
      const initialAdminBalance = await stableCoin.balanceOf(admin.address);
      const initialUser1Balance = await stableCoin.balanceOf(user1.address);
      const initialUser2Balance = await stableCoin.balanceOf(user2.address);
      
      // 2. Transfer from user1 to user2
      await stableCoin.connect(user1).transfer(user2.address, transferAmount);
      
      // 3. Check balances after transfer
      expect(await stableCoin.balanceOf(user1.address)).to.equal(initialUser1Balance - transferAmount);
      expect(await stableCoin.balanceOf(user2.address)).to.equal(initialUser2Balance + transferAmount);
      
      // 4. Verify cooldown is enforced - try to transfer again immediately
      await expect(
        stableCoin.connect(user1).transfer(user2.address, transferAmount)
      ).to.be.reverted; // Will revert with cooldown error
      
      // 5. Wait for cooldown to expire
      await time.increase(61); // 61 seconds
      
      // 6. Now transfer should work
      await stableCoin.connect(user1).transfer(user2.address, transferAmount);
      
      // 7. Check updated balances
      expect(await stableCoin.balanceOf(user1.address)).to.equal(initialUser1Balance - (transferAmount * 2n));
      expect(await stableCoin.balanceOf(user2.address)).to.equal(initialUser2Balance + (transferAmount * 2n));
    });
    
    it("should enforce whitelist restrictions", async function () {
      // 1. Try to transfer to non-whitelisted user3
      const transferAmount = ethers.parseUnits("1000", 18);
      
      await expect(
        stableCoin.connect(user1).transfer(user3.address, transferAmount)
      ).to.be.revertedWithCustomError(stableCoin, "NotWhitelisted");
      
      // 2. Whitelist user3
      await whitelist.setWhitelisted(user3.address, true);
      
      // 3. Now transfer should succeed
      await stableCoin.connect(user1).transfer(user3.address, transferAmount);
      expect(await stableCoin.balanceOf(user3.address)).to.equal(transferAmount);
    });
    it("should respect period limits over multiple days", async function () {
      const stableCoinAddress = await stableCoin.getAddress();
      
      // Temporarily increase max transfer limit for this test
      await transferLimiter.setDefaultMaxTransferAmount(
        stableCoinAddress, 
        ethers.parseUnits("100000", 18) // Much higher than period limit
      );
      
      // 1. Make transfers up to period limit
      const periodLimit = ethers.parseUnits("50000", 18);
      
      // Ensure user1 has enough tokens
      const user1Balance = await stableCoin.balanceOf(user1.address);
      if (user1Balance < periodLimit) {
        await stableCoin.transfer(user1.address, periodLimit - user1Balance);
      }
      
      // Now we can transfer the full amount at once
      await stableCoin.connect(user1).transfer(user2.address, periodLimit);
      
      // 2. Try to transfer more - should fail
      const smallAmount = ethers.parseUnits("1", 18);
      await stableCoin.transfer(user1.address, smallAmount);
      
      await expect(
        stableCoin.connect(user1).transfer(user2.address, smallAmount)
      ).to.be.reverted;
      
      // 3. Wait for a day to pass
      await time.increase(86401); // 24 hours + 1 second
      
      // 4. Now transfer should work (new period)
      await stableCoin.connect(user1).transfer(user2.address, smallAmount);
      expect(await stableCoin.balanceOf(user2.address)).to.equal(periodLimit + smallAmount);
      
      // Restore original limit if needed
      await transferLimiter.setDefaultMaxTransferAmount(
        stableCoinAddress, 
        ethers.parseUnits("10000", 18)
      );
    });
    
    it("should allow system-wide configuration changes", async function () {
      // 1. Disable limit checks
      await stableCoin.updateConfig(false, true);
      
      // Transfer large amount should now work
      const largeAmount = ethers.parseUnits("15000", 18);
      
      // Ensure user1 has enough tokens
      const user1Balance = await stableCoin.balanceOf(user1.address);
      if (user1Balance < largeAmount) {
        await stableCoin.transfer(user1.address, largeAmount - user1Balance);
      }
      
      await stableCoin.connect(user1).transfer(user2.address, largeAmount);
      
      // 2. Re-enable limit checks but disable whitelist
      await stableCoin.updateConfig(true, false);
      
      // Transfer to non-whitelisted user3 should work
      const amount = ethers.parseUnits("1000", 18);
      await stableCoin.connect(user1).transfer(user3.address, amount);
      expect(await stableCoin.balanceOf(user3.address)).to.equal(amount);
      
      // 3. Re-enable both checks
      await stableCoin.updateConfig(true, true);
      
      // Now transfer to non-whitelisted user should fail
      await expect(
        stableCoin.connect(user1).transfer(user3.address, amount)
      ).to.be.revertedWithCustomError(stableCoin, "NotWhitelisted");
    });
    
    it("should handle recovery from paused state correctly", async function () {
      // 1. Pause the contract
      const DEFAULT_ADMIN_ROLE = await stableCoin.DEFAULT_ADMIN_ROLE();
      const PAUSER_ROLE = ethers.keccak256(ethers.toUtf8Bytes("PAUSER_ROLE"));
      
      // Grant pauser role if needed
      if (!(await stableCoin.hasRole(PAUSER_ROLE, admin.address))) {
        await stableCoin.grantRole(PAUSER_ROLE, admin.address);
      }
      
      await stableCoin.pause();
      
      // 2. Verify transfers don't work while paused
      const transferAmount = ethers.parseUnits("1000", 18);
      await expect(
        stableCoin.connect(user1).transfer(user2.address, transferAmount)
      ).to.be.reverted;
      
      // 3. Unpause
      await stableCoin.unpause();
      
      // 4. Verify transfers work again
      await stableCoin.connect(user1).transfer(user2.address, transferAmount);
      expect(await stableCoin.balanceOf(user2.address)).to.equal(transferAmount);
    });
  });
  
  describe("Complete System Upgrades", function () {
    it("should handle replacing whitelist and limiter contracts", async function () {
      // 1. Deploy new utility contracts
      const WhitelistFactory = await ethers.getContractFactory("Whitelist");
      const newWhitelist = await WhitelistFactory.deploy(true) as Whitelist;
      await newWhitelist.waitForDeployment();
      
      const TransferLimiterFactory = await ethers.getContractFactory("TransferLimiter");
      const newTransferLimiter = await TransferLimiterFactory.deploy() as TransferLimiter;
      await newTransferLimiter.waitForDeployment();
      
      // 2. Set up the new contracts
      const stableCoinAddress = await stableCoin.getAddress();
      
      // Set up new whitelist
      await newWhitelist.batchSetWhitelisted([user1.address, user2.address], true);
      await newWhitelist.authorizeContract(stableCoinAddress);
      
      // Set up new transfer limiter
      const limitConfig = {
        maxTransferAmount: ethers.parseUnits("5000", 18), // Different limit than before
        cooldownPeriod: 30n,  // 30 second cooldown (different than before)
        periodLimit: ethers.parseUnits("25000", 18), // Different limit than before
        periodDuration: 86400n // 1 day period
      };
      await newTransferLimiter.setAllDefaultLimits(stableCoinAddress, limitConfig);
      await newTransferLimiter.authorizeContract(stableCoinAddress);
      await newTransferLimiter.setExemption(stableCoinAddress, admin.address, true);
      
      // 3. Update StableCoin to use new services
      await stableCoin.setWhitelistManager(await newWhitelist.getAddress());
      await stableCoin.setTransferLimiter(await newTransferLimiter.getAddress());
      
      // 4. Verify the new limits are enforced
      const smallAmount = ethers.parseUnits("1000", 18);
      await stableCoin.connect(user1).transfer(user2.address, smallAmount); // Should work
      
      const exceedingAmount = ethers.parseUnits("6000", 18); // Exceeds new 5000 limit
      await expect(
        stableCoin.connect(user1).transfer(user2.address, exceedingAmount)
      ).to.be.revertedWithCustomError(stableCoin, "LimitExceeded");
      
      // 5. Verify new cooldown is enforced
      await time.increase(31); // 31 seconds (just over new cooldown)
      await stableCoin.connect(user1).transfer(user2.address, smallAmount); // Should work now
    });
  });
});