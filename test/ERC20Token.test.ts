import { expect } from "chai";
import { ethers } from "hardhat";
import { ERC20Token } from "../typechain";
import { SignerWithAddress } from "@nomicfoundation/hardhat-ethers/signers";

describe("ERC20Token Contract", function () {
  let token: ERC20Token;
  let factorySigner: SignerWithAddress;
  let owner: SignerWithAddress;
  let user1: SignerWithAddress;
  let user2: SignerWithAddress;
  
  const NAME = "Test Token";
  const SYMBOL = "TT";
  
  // Role constants
  const ADMIN_ROLE = ethers.keccak256(ethers.toUtf8Bytes("ADMIN_ROLE"));
  const PAUSER_ROLE = ethers.keccak256(ethers.toUtf8Bytes("PAUSER_ROLE"));
  
  beforeEach(async function () {
    [factorySigner, owner, user1, user2] = await ethers.getSigners();
    
    const ERC20TokenFactory = await ethers.getContractFactory("ERC20Token");
    token = await ERC20TokenFactory.connect(factorySigner).deploy(NAME, SYMBOL, owner.address);
  });
  
  describe("Deployment", function () {
    it("Should set the right name and symbol", async function () {
      expect(await token.name()).to.equal(NAME);
      expect(await token.symbol()).to.equal(SYMBOL);
    });
    
    it("Should set the factory address correctly", async function () {
      expect(await token.factory()).to.equal(factorySigner.address);
    });
    
    it("Should assign the right roles to the token owner", async function () {
      expect(await token.hasRole(ethers.ZeroHash, owner.address)).to.be.true; // DEFAULT_ADMIN_ROLE
      expect(await token.hasRole(ADMIN_ROLE, owner.address)).to.be.true;
      expect(await token.hasRole(PAUSER_ROLE, owner.address)).to.be.true;
    });
    
    it("Should revert if token owner is zero address", async function () {
      const ERC20TokenFactory = await ethers.getContractFactory("ERC20Token");
      await expect(ERC20TokenFactory.deploy(NAME, SYMBOL, ethers.ZeroAddress))
        .to.be.revertedWith("Token owner cannot be zero address");
    });
  });
  
  describe("Mint functionality", function () {
    it("Should allow factory to mint tokens", async function () {
      const mintAmount = 1000n;
      await token.connect(factorySigner).mint(user1.address, mintAmount);
      expect(await token.balanceOf(user1.address)).to.equal(mintAmount);
    });
    
    it("Should not allow non-factory addresses to mint", async function () {
      await expect(token.connect(owner).mint(user1.address, 1000))
        .to.be.revertedWith("Only factory can mint");
    });
    
    it("Should revert if minting to zero address", async function () {
      await expect(token.connect(factorySigner).mint(ethers.ZeroAddress, 1000))
        .to.be.revertedWith("Cannot mint to zero address");
    });
    
    it("Should revert if amount is zero", async function () {
      await expect(token.connect(factorySigner).mint(user1.address, 0))
        .to.be.revertedWith("Amount must be greater than zero");
    });
  });
  
  describe("Burn functionality", function () {
    beforeEach(async function () {
      // Mint some tokens to the factory for burning
      await token.connect(factorySigner).mint(factorySigner.address, 10000);
    });
    
    it("Should allow factory to burn its own tokens", async function () {
      const initialBalance = await token.balanceOf(factorySigner.address);
      const burnAmount = 1000n;
      
      await token.connect(factorySigner).burn(burnAmount);
      expect(await token.balanceOf(factorySigner.address)).to.equal(initialBalance - burnAmount);
    });
    
    it("Should not allow non-factory addresses to burn", async function () {
      await expect(token.connect(owner).burn(1000))
        .to.be.revertedWith("Only factory can burn");
    });
    
    it("Should revert if amount is zero", async function () {
      await expect(token.connect(factorySigner).burn(0))
        .to.be.revertedWith("Amount must be greater than zero");
    });
    
    it("Should revert if burning more than balance", async function () {
      const balance = await token.balanceOf(factorySigner.address);
      await expect(token.connect(factorySigner).burn(balance + 1n))
        .to.be.revertedWith("Insufficient balance");
    });
  });
  
  describe("BurnFrom functionality", function () {
    beforeEach(async function () {
      // Mint some tokens to user1
      await token.connect(factorySigner).mint(user1.address, 10000);
      // Approve factory to spend user1's tokens
      await token.connect(user1).approve(factorySigner.address, 5000);
    });
    
    it("Should allow factory to burn from approved account", async function () {
      const initialBalance = await token.balanceOf(user1.address);
      const burnAmount = 2000n;
      
      await token.connect(factorySigner).burnFrom(user1.address, burnAmount);
      expect(await token.balanceOf(user1.address)).to.equal(initialBalance - burnAmount);
    });
    
    it("Should reduce allowance after burnFrom", async function () {
      const initialAllowance = await token.allowance(user1.address, factorySigner.address);
      const burnAmount = 2000n;
      
      await token.connect(factorySigner).burnFrom(user1.address, burnAmount);
      expect(await token.allowance(user1.address, factorySigner.address))
        .to.equal(initialAllowance - burnAmount);
    });
    
    it("Should not allow burning more than allowance", async function () {
      const allowance = await token.allowance(user1.address, factorySigner.address);
      await expect(token.connect(factorySigner).burnFrom(user1.address, allowance + 1n))
        .to.be.revertedWith("ERC20: burn amount exceeds allowance");
    });
    
    it("Should not allow non-factory addresses to burnFrom", async function () {
      await expect(token.connect(owner).burnFrom(user1.address, 1000))
        .to.be.revertedWith("Only factory can burn");
    });
  });
  
  describe("Pause functionality", function () {
    beforeEach(async function () {
      await token.connect(factorySigner).mint(user1.address, 10000);
      await token.connect(user1).approve(user2.address, 5000);
    });
    
    it("Should pause all transfers when paused", async function () {
      await token.connect(owner).pause();
      await expect(token.connect(user1).transfer(user2.address, 1000))
        .to.be.reverted;
    });
    
    it("Should pause mint operations when paused", async function () {
      await token.connect(owner).pause();
      await expect(token.connect(factorySigner).mint(user1.address, 1000))
        .to.be.reverted;
    });
    
    it("Should pause burn operations when paused", async function () {
      await token.connect(factorySigner).mint(factorySigner.address, 10000);
      await token.connect(owner).pause();
      await expect(token.connect(factorySigner).burn(1000))
        .to.be.reverted;
    });
    
    it("Should resume operations when unpaused", async function () {
      await token.connect(owner).pause();
      await token.connect(owner).unpause();
      await expect(token.connect(user1).transfer(user2.address, 1000))
        .to.not.be.reverted;
    });
    
    it("Should only allow pausers to pause/unpause", async function () {
      await expect(token.connect(user1).pause())
        .to.be.revertedWithCustomError(token, "AccessControlUnauthorizedAccount")
        .withArgs(user1.address, PAUSER_ROLE);
      
      await token.connect(owner).pause();
      
      await expect(token.connect(user1).unpause())
        .to.be.revertedWithCustomError(token, "AccessControlUnauthorizedAccount")
        .withArgs(user1.address, PAUSER_ROLE);
    });
  });
});