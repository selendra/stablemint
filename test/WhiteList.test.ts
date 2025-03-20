import { expect } from "chai";
import { ethers } from "hardhat";
import { Contract, ZeroAddress } from "ethers";
import { HardhatEthersSigner } from "@nomicfoundation/hardhat-ethers/signers";

describe("Whitelist Contract", function () {
  let whitelist: any;
  let owner: HardhatEthersSigner;
  let admin: HardhatEthersSigner;
  let whitelister: HardhatEthersSigner;
  let user1: HardhatEthersSigner;
  let user2: HardhatEthersSigner;
  let authorizedContract: HardhatEthersSigner;

  // Role constants
  const ADMIN_ROLE = ethers.keccak256(ethers.toUtf8Bytes("ADMIN_ROLE"));
  const WHITELISTER_ROLE = ethers.keccak256(ethers.toUtf8Bytes("WHITELISTER_ROLE"));
  const CONTRACT_ROLE = ethers.keccak256(ethers.toUtf8Bytes("CONTRACT_ROLE"));
  const DEFAULT_ADMIN_ROLE = "0x0000000000000000000000000000000000000000000000000000000000000000";

  beforeEach(async function () {
    // Get signers
    [owner, admin, whitelister, user1, user2, authorizedContract] = await ethers.getSigners();
    
    // Deploy the contract
    const WhitelistFactory = await ethers.getContractFactory("Whitelist");
    whitelist = await WhitelistFactory.deploy();
    await whitelist.waitForDeployment();
  });

  describe("Deployment", function () {
    it("Should set the deployer as admin and whitelister", async function () {
      expect(await whitelist.hasRole(DEFAULT_ADMIN_ROLE, owner.address)).to.equal(true);
      expect(await whitelist.hasRole(ADMIN_ROLE, owner.address)).to.equal(true);
      expect(await whitelist.hasRole(WHITELISTER_ROLE, owner.address)).to.equal(true);
    });
    
    it("Should add deployer to whitelist", async function () {
      expect(await whitelist.isWhitelisted(owner.address)).to.equal(true);
    });
    
    it("Should have whitelisting disabled by default", async function () {
      expect(await whitelist.whitelistingEnabled()).to.equal(false);
    });
  });
  
  describe("Role Management", function () {
    it("Should allow admin to add another admin", async function () {
      await whitelist.grantRole(ADMIN_ROLE, admin.address);
      expect(await whitelist.hasRole(ADMIN_ROLE, admin.address)).to.equal(true);
    });
    
    it("Should allow admin to add a whitelister", async function () {
      await whitelist.addWhitelister(whitelister.address);
      expect(await whitelist.hasRole(WHITELISTER_ROLE, whitelister.address)).to.equal(true);
    });
    
    it("Should allow admin to remove a whitelister", async function () {
      await whitelist.addWhitelister(whitelister.address);
      await whitelist.removeWhitelister(whitelister.address);
      expect(await whitelist.hasRole(WHITELISTER_ROLE, whitelister.address)).to.equal(false);
    });
    
    it("Should allow admin to authorize a contract", async function () {
      await whitelist.authorizeContract(authorizedContract.address);
      expect(await whitelist.hasRole(CONTRACT_ROLE, authorizedContract.address)).to.equal(true);
    });
    
    it("Should revert when trying to authorize zero address as contract", async function () {
      await expect(whitelist.authorizeContract(ZeroAddress))
        .to.be.revertedWithCustomError(whitelist, "ZeroAddress");
    });
    
    it("Should revert when trying to add zero address as whitelister", async function () {
      await expect(whitelist.addWhitelister(ZeroAddress))
        .to.be.revertedWithCustomError(whitelist, "ZeroAddress");
    });
    
    it("Should revert when non-admin tries to add a whitelister", async function () {
      await expect(whitelist.connect(user1).addWhitelister(user2.address))
        .to.be.revertedWithCustomError(whitelist, "AccessControlUnauthorizedAccount")
        .withArgs(user1.address, ADMIN_ROLE);
    });
  });

  describe("Whitelist Management", function () {
    beforeEach(async function () {
      // Setup: Add whitelister role and enable whitelisting
      await whitelist.addWhitelister(whitelister.address);
      await whitelist.toggleWhitelisting(true);
    });
    
    it("Should allow whitelister to add an address to whitelist", async function () {
      await whitelist.connect(whitelister).setWhitelisted(user1.address, true);
      expect(await whitelist.isWhitelisted(user1.address)).to.equal(true);
    });
    
    it("Should allow whitelister to remove an address from whitelist", async function () {
      // First add to whitelist
      await whitelist.connect(whitelister).setWhitelisted(user1.address, true);
      // Then remove
      await whitelist.connect(whitelister).setWhitelisted(user1.address, false);
      expect(await whitelist.isWhitelisted(user1.address)).to.equal(false);
    });
    
    it("Should allow batch whitelisting", async function () {
      const addresses = [user1.address, user2.address];
      await whitelist.connect(whitelister).batchSetWhitelisted(addresses, true);
      
      expect(await whitelist.isWhitelisted(user1.address)).to.equal(true);
      expect(await whitelist.isWhitelisted(user2.address)).to.equal(true);
    });
    
    it("Should revert when non-whitelister tries to modify whitelist", async function () {
      await expect(whitelist.connect(user1).setWhitelisted(user2.address, true))
        .to.be.revertedWithCustomError(whitelist, "AccessControlUnauthorizedAccount")
        .withArgs(user1.address, WHITELISTER_ROLE);
    });
    
    it("Should emit WhitelistUpdated event when whitelist is modified", async function () {
      await expect(whitelist.connect(whitelister).setWhitelisted(user1.address, true))
        .to.emit(whitelist, "WhitelistUpdated")
        .withArgs(user1.address, true);
    });
  });

  describe("Whitelist Toggling", function () {
    it("Should allow admin to enable whitelisting", async function () {
      await whitelist.toggleWhitelisting(true);
      expect(await whitelist.whitelistingEnabled()).to.equal(true);
    });
    
    it("Should allow admin to disable whitelisting", async function () {
      // First enable
      await whitelist.toggleWhitelisting(true);
      // Then disable
      await whitelist.toggleWhitelisting(false);
      expect(await whitelist.whitelistingEnabled()).to.equal(false);
    });
    
    it("Should emit WhitelistingToggled event", async function () {
      await expect(whitelist.toggleWhitelisting(true))
        .to.emit(whitelist, "WhitelistingToggled")
        .withArgs(true);
    });
    
    it("Should revert when non-admin tries to toggle whitelisting", async function () {
      await expect(whitelist.connect(user1).toggleWhitelisting(true))
        .to.be.revertedWithCustomError(whitelist, "AccessControlUnauthorizedAccount")
        .withArgs(user1.address, ADMIN_ROLE);
    });
  });

  describe("Whitelist Checking", function () {
    beforeEach(async function () {
      // Setup: Enable whitelisting
      await whitelist.toggleWhitelisting(true);
      // Add user1 to whitelist
      await whitelist.setWhitelisted(user1.address, true);
    });
    
    it("Should return true for whitelisted addresses", async function () {
      expect(await whitelist.isWhitelisted(user1.address)).to.equal(true);
      expect(await whitelist.checkWhitelist(user1.address)).to.equal(true);
    });
    
    it("Should return false for non-whitelisted addresses", async function () {
      expect(await whitelist.isWhitelisted(user2.address)).to.equal(false);
      expect(await whitelist.checkWhitelist(user2.address)).to.equal(false);
    });
    
    it("Should return true for all addresses when whitelisting is disabled", async function () {
      // Disable whitelisting
      await whitelist.toggleWhitelisting(false);
      
      expect(await whitelist.isWhitelisted(user1.address)).to.equal(true);
      expect(await whitelist.isWhitelisted(user2.address)).to.equal(true);
      expect(await whitelist.checkWhitelist(user1.address)).to.equal(true);
      expect(await whitelist.checkWhitelist(user2.address)).to.equal(true);
    });
  });
}); 
