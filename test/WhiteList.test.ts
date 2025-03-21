import { expect } from "chai";
import { ethers } from "hardhat";
import { HardhatEthersSigner } from "@nomicfoundation/hardhat-ethers/signers";
import { Whitelist } from "../typechain-types";

describe("Whitelist", function () {
  let whitelist: Whitelist;
  let admin: HardhatEthersSigner, whitelister: HardhatEthersSigner, user1: HardhatEthersSigner, 
      user2: HardhatEthersSigner, user3: HardhatEthersSigner, contractMock: HardhatEthersSigner;
  
  beforeEach(async function () {
    [admin, whitelister, user1, user2, user3, contractMock] = await ethers.getSigners();
    
    const Whitelist = await ethers.getContractFactory("Whitelist");
    whitelist = await Whitelist.deploy(true) as Whitelist; // Enable whitelisting
    await whitelist.waitForDeployment();
  });
  
  describe("Initialization", function () {
    it("should set deployer as admin and whitelister", async function () {
      const DEFAULT_ADMIN_ROLE = await whitelist.DEFAULT_ADMIN_ROLE();
      const ADMIN_ROLE = await whitelist.ADMIN_ROLE();
      const WHITELISTER_ROLE = await whitelist.WHITELISTER_ROLE();
      
      expect(await whitelist.hasRole(DEFAULT_ADMIN_ROLE, admin.address)).to.be.true;
      expect(await whitelist.hasRole(ADMIN_ROLE, admin.address)).to.be.true;
      expect(await whitelist.hasRole(WHITELISTER_ROLE, admin.address)).to.be.true;
    });
    
    it("should enable whitelisting by default", async function () {
      expect(await whitelist.whitelistingEnabled()).to.be.true;
    });
    
    it("should whitelist the deployer by default", async function () {
      expect(await whitelist.isWhitelisted(admin.address)).to.be.true;
    });
  });
  
  describe("Role Management", function () {
    it("should allow admin to add a whitelister", async function () {
      const WHITELISTER_ROLE = await whitelist.WHITELISTER_ROLE();
      await whitelist.addWhitelister(whitelister.address);
      expect(await whitelist.hasRole(WHITELISTER_ROLE, whitelister.address)).to.be.true;
    });
    
    it("should allow admin to remove a whitelister", async function () {
      const WHITELISTER_ROLE = await whitelist.WHITELISTER_ROLE();
      await whitelist.addWhitelister(whitelister.address);
      await whitelist.removeWhitelister(whitelister.address);
      expect(await whitelist.hasRole(WHITELISTER_ROLE, whitelister.address)).to.be.false;
    });
    
    it("should prevent non-admin from adding a whitelister", async function () {
      await expect(
        whitelist.connect(user1).addWhitelister(user2.address)
      ).to.be.reverted;
    });
  });
  
  describe("Whitelist Management", function () {
    beforeEach(async function () {
      await whitelist.addWhitelister(whitelister.address);
    });
    
    it("should allow whitelister to add an address to whitelist", async function () {
      await whitelist.connect(whitelister).setWhitelisted(user1.address, true);
      expect(await whitelist.isWhitelisted(user1.address)).to.be.true;
    });
    
    it("should allow whitelister to remove an address from whitelist", async function () {
      await whitelist.connect(whitelister).setWhitelisted(user1.address, true);
      await whitelist.connect(whitelister).setWhitelisted(user1.address, false);
      expect(await whitelist.isWhitelisted(user1.address)).to.be.false;
    });
    
    it("should prevent non-whitelister from modifying whitelist", async function () {
      await expect(
        whitelist.connect(user1).setWhitelisted(user2.address, true)
      ).to.be.reverted;
    });
    
    it("should allow batch whitelisting", async function () {
      const addresses = [user1.address, user2.address, user3.address];
      await whitelist.connect(whitelister).batchSetWhitelisted(addresses, true);
      
      for (const addr of addresses) {
        expect(await whitelist.isWhitelisted(addr)).to.be.true;
      }
    });
  });
  
  describe("Global Whitelist Control", function () {
    it("should allow admin to disable whitelisting", async function () {
      await whitelist.toggleWhitelisting(false);
      expect(await whitelist.whitelistingEnabled()).to.be.false;
      
      // When disabled, all addresses should pass whitelist check
      expect(await whitelist.isWhitelisted(user1.address)).to.be.true;
    });
    
    it("should allow admin to re-enable whitelisting", async function () {
      await whitelist.toggleWhitelisting(false);
      await whitelist.toggleWhitelisting(true);
      expect(await whitelist.whitelistingEnabled()).to.be.true;
      
      // When re-enabled, non-whitelisted addresses should fail check
      expect(await whitelist.isWhitelisted(user1.address)).to.be.false;
    });
  });
  
  describe("Contract Authorization", function () {
    it("should allow admin to authorize a contract", async function () {
      const CONTRACT_ROLE = await whitelist.CONTRACT_ROLE();
      await whitelist.authorizeContract(contractMock.address);
      expect(await whitelist.hasRole(CONTRACT_ROLE, contractMock.address)).to.be.true;
    });
    
    it("should prevent authorizing zero address as contract", async function () {
      await expect(
        whitelist.authorizeContract(ethers.ZeroAddress)
      ).to.be.revertedWithCustomError(whitelist, "ZeroAddress");
    });
  });
  
  describe("Events", function () {
    it("should emit WhitelistUpdated event when whitelist status changes", async function () {
      await expect(whitelist.setWhitelisted(user1.address, true))
        .to.emit(whitelist, "WhitelistUpdated")
        .withArgs(user1.address, true);
    });
    
    it("should emit WhitelistingToggled event when global status changes", async function () {
      await expect(whitelist.toggleWhitelisting(false))
        .to.emit(whitelist, "WhitelistingToggled")
        .withArgs(false);
    });
  });
});