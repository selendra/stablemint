import { expect } from "chai";
import { ethers } from "hardhat";
import { StableCoin } from "../typechain";
import { SignerWithAddress } from "@nomicfoundation/hardhat-ethers/signers";

describe("StableCoin Contract", function () {
  let stableCoin: StableCoin;
  let owner: SignerWithAddress;
  let admin: SignerWithAddress;
  let user1: SignerWithAddress;
  let user2: SignerWithAddress;
  let users: SignerWithAddress[];
  
  const NAME = "Test Stable Coin";
  const SYMBOL = "TSC";
  const INITIAL_SUPPLY = 1000000; // 1 million tokens
  
  // Role constants from the contract
  const PAUSER_ROLE = ethers.keccak256(ethers.toUtf8Bytes("PAUSER_ROLE"));
  const MINTER_ROLE = ethers.keccak256(ethers.toUtf8Bytes("MINTER_ROLE"));
  const BURNER_ROLE = ethers.keccak256(ethers.toUtf8Bytes("BURNER_ROLE"));
  const ADMIN_ROLE = ethers.keccak256(ethers.toUtf8Bytes("ADMIN_ROLE"));
  const WHITELIST_MANAGER_ROLE = ethers.keccak256(ethers.toUtf8Bytes("WHITELIST_MANAGER_ROLE"));
  
  beforeEach(async function () {
    [owner, admin, user1, user2, ...users] = await ethers.getSigners();
    
    const StableCoinFactory = await ethers.getContractFactory("StableCoin");
    stableCoin = await StableCoinFactory.deploy(NAME, SYMBOL, INITIAL_SUPPLY);
    
    // Grant roles to admin for testing
    await stableCoin.grantRole(ADMIN_ROLE, admin.address);
    await stableCoin.grantRole(WHITELIST_MANAGER_ROLE, admin.address);
    
    // *** IMPORTANT FIX: Add owner to whitelist first! ***
    await stableCoin.addToWhitelist(owner.address);
  });
  
  describe("Deployment", function () {
    it("Should set the right name and symbol", async function () {
      expect(await stableCoin.name()).to.equal(NAME);
      expect(await stableCoin.symbol()).to.equal(SYMBOL);
    });
    
    it("Should assign the initial supply to the owner", async function () {
      const decimals = await stableCoin.decimals();
      const expectedSupply = BigInt(INITIAL_SUPPLY) * (10n ** BigInt(decimals));
      expect(await stableCoin.totalSupply()).to.equal(expectedSupply);
      expect(await stableCoin.balanceOf(owner.address)).to.equal(expectedSupply);
    });
    
    it("Should set the right roles to owner", async function () {
      expect(await stableCoin.hasRole(ethers.ZeroHash, owner.address)).to.be.true; // DEFAULT_ADMIN_ROLE
      expect(await stableCoin.hasRole(ADMIN_ROLE, owner.address)).to.be.true;
      expect(await stableCoin.hasRole(PAUSER_ROLE, owner.address)).to.be.true;
      expect(await stableCoin.hasRole(MINTER_ROLE, owner.address)).to.be.true;
      expect(await stableCoin.hasRole(BURNER_ROLE, owner.address)).to.be.true;
      expect(await stableCoin.hasRole(WHITELIST_MANAGER_ROLE, owner.address)).to.be.true;
    });
    
    it("Should enforce whitelist for receivers by default", async function () {
      expect(await stableCoin.enforceWhitelistForReceivers()).to.be.true;
    });
  });
  
  describe("Whitelist Management", function () {
    it("Should add an address to the whitelist", async function () {
      await stableCoin.connect(admin).addToWhitelist(user1.address);
      expect(await stableCoin.whitelisted(user1.address)).to.be.true;
    });
    
    it("Should emit a Whitelisted event when adding to whitelist", async function () {
      await expect(stableCoin.connect(admin).addToWhitelist(user1.address))
        .to.emit(stableCoin, "Whitelisted")
        .withArgs(user1.address, true);
    });
    
    it("Should remove an address from the whitelist", async function () {
      await stableCoin.connect(admin).addToWhitelist(user1.address);
      await stableCoin.connect(admin).removeFromWhitelist(user1.address);
      expect(await stableCoin.whitelisted(user1.address)).to.be.false;
    });
    
    it("Should batch add addresses to the whitelist", async function () {
      const batchAddresses = [user1.address, user2.address, users[0].address];
      await stableCoin.connect(admin).batchAddToWhitelist(batchAddresses);
      
      for (const address of batchAddresses) {
        expect(await stableCoin.whitelisted(address)).to.be.true;
      }
    });
    
    it("Should not allow non-whitelist managers to add to whitelist", async function () {
      await expect(stableCoin.connect(user1).addToWhitelist(user2.address))
        .to.be.revertedWithCustomError(stableCoin, "AccessControlUnauthorizedAccount")
        .withArgs(user1.address, WHITELIST_MANAGER_ROLE);
    });
    
    it("Should allow changing whitelist receiver policy", async function () {
      await stableCoin.connect(admin).setWhitelistReceiverPolicy(false);
      expect(await stableCoin.enforceWhitelistForReceivers()).to.be.false;
      
      await stableCoin.connect(admin).setWhitelistReceiverPolicy(true);
      expect(await stableCoin.enforceWhitelistForReceivers()).to.be.true;
    });
  });
  
  describe("Transfer Restrictions", function () {
    beforeEach(async function () {
      const decimals = await stableCoin.decimals();
      const amount = BigInt(1000) * (10n ** BigInt(decimals));
      
      // Add user1 to whitelist and transfer some tokens
      await stableCoin.connect(admin).addToWhitelist(user1.address);
      await stableCoin.connect(owner).transfer(user1.address, amount);
    });
    
    it("Should not allow transfers from non-whitelisted addresses", async function () {
      // User2 is not whitelisted
      await expect(stableCoin.connect(user2).transfer(user1.address, 100))
        .to.be.revertedWith("Sender not whitelisted");
    });
    
    it("Should not allow transfers to non-whitelisted addresses when policy enforced", async function () {
      // User2 is not whitelisted
      await expect(stableCoin.connect(user1).transfer(user2.address, 100))
        .to.be.revertedWith("Receiver not whitelisted");
    });
    
    it("Should allow transfers to non-whitelisted addresses when policy not enforced", async function () {
      // Disable receiver whitelist policy
      await stableCoin.connect(admin).setWhitelistReceiverPolicy(false);
      
      // Now transfer should work even though user2 is not whitelisted
      await expect(stableCoin.connect(user1).transfer(user2.address, 100))
        .to.not.be.reverted;
    });
    
    it("Should allow transfers between whitelisted addresses", async function () {
      await stableCoin.connect(admin).addToWhitelist(user2.address);
      await expect(stableCoin.connect(user1).transfer(user2.address, 100))
        .to.not.be.reverted;
    });
  });
  
  describe("Pause Functionality", function () {
    beforeEach(async function () {
      await stableCoin.connect(admin).addToWhitelist(user1.address);
      await stableCoin.connect(admin).addToWhitelist(user2.address);
      await stableCoin.connect(owner).transfer(user1.address, 1000);
    });
    
    it("Should pause all transfers when paused", async function () {
      await stableCoin.connect(owner).pause();
      await expect(stableCoin.connect(user1).transfer(user2.address, 100))
        .to.be.reverted;
    });
    
    it("Should resume transfers when unpaused", async function () {
      await stableCoin.connect(owner).pause();
      await stableCoin.connect(owner).unpause();
      await expect(stableCoin.connect(user1).transfer(user2.address, 100))
        .to.not.be.reverted;
    });
    
    it("Should only allow pausers to pause", async function () {
      await expect(stableCoin.connect(user1).pause())
        .to.be.revertedWithCustomError(stableCoin, "AccessControlUnauthorizedAccount")
        .withArgs(user1.address, PAUSER_ROLE);
    });
  });
  
  describe("Mint and Burn", function () {
    beforeEach(async function () {
      await stableCoin.connect(admin).addToWhitelist(user1.address);
    });
    
    it("Should allow minters to mint new tokens", async function () {
      const initialBalance = await stableCoin.balanceOf(user1.address);
      const mintAmount = 1000n;
      
      await stableCoin.connect(owner).mint(user1.address, mintAmount);
      
      expect(await stableCoin.balanceOf(user1.address)).to.equal(initialBalance + mintAmount);
    });
    
    it("Should not allow non-minters to mint tokens", async function () {
      await expect(stableCoin.connect(user1).mint(user1.address, 1000))
        .to.be.revertedWithCustomError(stableCoin, "AccessControlUnauthorizedAccount")
        .withArgs(user1.address, MINTER_ROLE);
    });
    
    it("Should allow burners to burn their tokens", async function () {
      // First mint some tokens to the burner
      await stableCoin.connect(owner).mint(owner.address, 1000);
      const initialBalance = await stableCoin.balanceOf(owner.address);
      const burnAmount = 500n;
      
      await stableCoin.connect(owner).burn(burnAmount);
      
      expect(await stableCoin.balanceOf(owner.address)).to.equal(initialBalance - burnAmount);
    });
    
    it("Should not allow non-burners to burn tokens", async function () {
      await expect(stableCoin.connect(user1).burn(1000))
        .to.be.revertedWithCustomError(stableCoin, "AccessControlUnauthorizedAccount")
        .withArgs(user1.address, BURNER_ROLE);
    });
  });
});