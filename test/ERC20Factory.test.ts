import { expect } from "chai";
import { ethers } from "hardhat";
import { StableCoin, ERC20Factory, ERC20Token } from "../typechain";
import { SignerWithAddress } from "@nomicfoundation/hardhat-ethers/signers";

describe("ERC20Factory Contract", function () {
  let stableCoin: StableCoin;
  let factory: ERC20Factory;
  let owner: SignerWithAddress;
  let tokenCreator: SignerWithAddress;
  let factoryMinter: SignerWithAddress;
  let tokenOwner: SignerWithAddress;
  let user: SignerWithAddress;
  
  // Role constants
  const FACTORY_ADMIN_ROLE = ethers.keccak256(ethers.toUtf8Bytes("FACTORY_ADMIN_ROLE"));
  const TOKEN_CREATOR_ROLE = ethers.keccak256(ethers.toUtf8Bytes("TOKEN_CREATOR_ROLE"));
  const FACTORY_MINTER_ROLE = ethers.keccak256(ethers.toUtf8Bytes("FACTORY_MINTER_ROLE"));
  const RATIO_MANAGER_ROLE = ethers.keccak256(ethers.toUtf8Bytes("RATIO_MANAGER_ROLE"));
  
  beforeEach(async function () {
    [owner, tokenCreator, factoryMinter, tokenOwner, user] = await ethers.getSigners();
    
    // Deploy StableCoin
    const StableCoinFactory = await ethers.getContractFactory("StableCoin");
    stableCoin = await StableCoinFactory.deploy("Test Stable Coin", "TSC", 1000000);
    
    // Deploy Factory
    const ERC20FactoryContract = await ethers.getContractFactory("ERC20Factory");
    factory = await ERC20FactoryContract.deploy(owner.address, await stableCoin.getAddress());
    
    // Set up roles
    await factory.grantRole(TOKEN_CREATOR_ROLE, tokenCreator.address);
    await factory.grantRole(FACTORY_MINTER_ROLE, factoryMinter.address);
    
    // Mint and whitelist for factoryMinter
    const MINTER_ROLE = ethers.keccak256(ethers.toUtf8Bytes("MINTER_ROLE"));
    const WHITELIST_MANAGER_ROLE = ethers.keccak256(ethers.toUtf8Bytes("WHITELIST_MANAGER_ROLE"));
    
    await stableCoin.grantRole(MINTER_ROLE, owner.address);
    await stableCoin.grantRole(WHITELIST_MANAGER_ROLE, owner.address);
    await stableCoin.addToWhitelist(factoryMinter.address);
    await stableCoin.mint(factoryMinter.address, 10000);
  });
  
  describe("Deployment", function () {
    it("Should set the StableCoin address", async function () {
      expect(await factory.stableCoin()).to.equal(await stableCoin.getAddress());
    });
    
    it("Should assign correct roles to owner", async function () {
      expect(await factory.hasRole(ethers.ZeroHash, owner.address)).to.be.true; // DEFAULT_ADMIN_ROLE
      expect(await factory.hasRole(FACTORY_ADMIN_ROLE, owner.address)).to.be.true;
      expect(await factory.hasRole(TOKEN_CREATOR_ROLE, owner.address)).to.be.true;
      expect(await factory.hasRole(FACTORY_MINTER_ROLE, owner.address)).to.be.true;
      expect(await factory.hasRole(RATIO_MANAGER_ROLE, owner.address)).to.be.true;
    });
    
    it("Should revert if owner is zero address", async function () {
      const ERC20FactoryContract = await ethers.getContractFactory("ERC20Factory");
      await expect(ERC20FactoryContract.deploy(ethers.ZeroAddress, await stableCoin.getAddress()))
        .to.be.revertedWith("Factory owner cannot be zero address");
    });
    
    it("Should revert if stableCoin is zero address", async function () {
      const ERC20FactoryContract = await ethers.getContractFactory("ERC20Factory");
      await expect(ERC20FactoryContract.deploy(owner.address, ethers.ZeroAddress))
        .to.be.revertedWith("StableCoin address cannot be zero");
    });
  });
  
  describe("Token Creation", function () {
    it("Should create a new token with correct parameters", async function () {
      const tokenName = "Test Token";
      const tokenSymbol = "TT";
      const tokensPerStableCoin = 10n;
      
      const tx = await factory.connect(tokenCreator).createToken(
        tokenName,
        tokenSymbol,
        tokenOwner.address,
        tokensPerStableCoin
      );
      
      const receipt = await tx.wait();
      if (!receipt || !receipt.logs) {
        throw new Error("Transaction receipt or logs not available");
      }
      
      // Find the TokenCreated event to get the token address
      const event: any = receipt.logs.find(
        (log: any) => log.fragment && log.fragment.name === 'TokenCreated'
      );
      
      expect(event).to.not.be.undefined;
      
      if (!event || !event.args) {
        throw new Error("Event or args not found");
      }
      
      const tokenAddress = event.args[1];
      
      // Check if the token is registered in the factory
      expect(await factory.isTokenCreatedByFactory(tokenAddress)).to.be.true;
      
      // Check the token ratio
      expect(await factory.tokenRatios(tokenAddress)).to.equal(tokensPerStableCoin);
      
      // Verify token details directly
      const token = await ethers.getContractAt("ERC20Token", tokenAddress);
      expect(await token.name()).to.equal(tokenName);
      expect(await token.symbol()).to.equal(tokenSymbol);
      expect(await token.factory()).to.equal(await factory.getAddress());
      
      // Verify token owner has admin role
      const ADMIN_ROLE = ethers.keccak256(ethers.toUtf8Bytes("ADMIN_ROLE"));
      expect(await token.hasRole(ADMIN_ROLE, tokenOwner.address)).to.be.true;
    });
    
    it("Should emit TokenCreated and TokenRatioSet events", async function () {
      const tokenName = "Test Token";
      const tokenSymbol = "TT";
      const tokensPerStableCoin = 10n;
      
      await expect(factory.connect(tokenCreator).createToken(
        tokenName,
        tokenSymbol,
        tokenOwner.address,
        tokensPerStableCoin
      ))
        .to.emit(factory, "TokenRatioSet")
        .to.emit(factory, "TokenCreated");
    });
    
    it("Should revert if non-creator tries to create a token", async function () {
      await expect(factory.connect(user).createToken(
        "Test Token",
        "TT",
        tokenOwner.address,
        10
      ))
        .to.be.revertedWithCustomError(factory, "AccessControlUnauthorizedAccount")
        .withArgs(user.address, TOKEN_CREATOR_ROLE);
    });
    
    it("Should revert with empty name or symbol", async function () {
      await expect(factory.connect(tokenCreator).createToken(
        "",
        "TT",
        tokenOwner.address,
        10
      ))
        .to.be.revertedWith("Token name cannot be empty");
      
      await expect(factory.connect(tokenCreator).createToken(
        "Test Token",
        "",
        tokenOwner.address,
        10
      ))
        .to.be.revertedWith("Token symbol cannot be empty");
    });
    
    it("Should revert if token owner is zero address", async function () {
      await expect(factory.connect(tokenCreator).createToken(
        "Test Token",
        "TT",
        ethers.ZeroAddress,
        10
      ))
        .to.be.revertedWith("Cannot grant role to zero address");
    });
    
    it("Should revert if tokens per stablecoin is zero", async function () {
      await expect(factory.connect(tokenCreator).createToken(
        "Test Token",
        "TT",
        tokenOwner.address,
        0
      ))
        .to.be.revertedWith("Tokens per StableCoin must be greater than zero");
    });
  });
  
  describe("Token Minting", function () {
    let tokenAddress: string;
    const tokensPerStableCoin = 10n;
    
    beforeEach(async function () {
      // Create a token first
      const tx = await factory.connect(tokenCreator).createToken(
        "Test Token",
        "TT",
        tokenOwner.address,
        tokensPerStableCoin
      );
      
      const receipt = await tx.wait();
      if (!receipt || !receipt.logs) {
        throw new Error("Transaction receipt or logs not available");
      }
      
      const event: any = receipt.logs.find(
        (log: any) => log.fragment && log.fragment.name === 'TokenCreated'
      );
      
      if (!event || !event.args) {
        throw new Error("Event or args not found");
      }
      
      tokenAddress = event.args[1];
    });
    
    it("Should mint tokens through the factory", async function () {
      const mintAmount = 100n;
      
      await expect(factory.connect(factoryMinter).mintToken(
        tokenAddress,
        user.address,
        mintAmount
      ))
        .to.emit(factory, "TokenMinted")
        .withArgs(tokenAddress, user.address, mintAmount);
      
      const token = await ethers.getContractAt("ERC20Token", tokenAddress);
      expect(await token.balanceOf(user.address)).to.equal(mintAmount);
    });
    
    it("Should revert if token not created by factory", async function () {
      // Deploy a token directly, not through factory
      const ERC20TokenFactory = await ethers.getContractFactory("ERC20Token");
      const directToken = await ERC20TokenFactory.deploy("Direct Token", "DT", owner.address);
      
      await expect(factory.connect(factoryMinter).mintToken(
        await directToken.getAddress(),
        user.address,
        100
      ))
        .to.be.revertedWith("Token not created by this factory");
    });
    
    it("Should revert if non-minter tries to mint tokens", async function () {
      await expect(factory.connect(user).mintToken(
        tokenAddress,
        user.address,
        100
      ))
        .to.be.revertedWithCustomError(factory, "AccessControlUnauthorizedAccount")
        .withArgs(user.address, FACTORY_MINTER_ROLE);
    });
    
    it("Should revert if insufficient StableCoin balance", async function () {
      // Try to mint more tokens than the stablecoin balance can support
      const stableCoinBalance = await stableCoin.balanceOf(factoryMinter.address);
      const excessiveAmount = (stableCoinBalance + 1n) * tokensPerStableCoin;
      
      await expect(factory.connect(factoryMinter).mintToken(
        tokenAddress,
        user.address,
        excessiveAmount
      ))
        .to.be.revertedWith("Insufficient StableCoin balance for minting");
    });
  });
  
  describe("StableCoin Management", function () {
    it("Should allow changing StableCoin address", async function () {
      // Deploy a new StableCoin
      const StableCoinFactory = await ethers.getContractFactory("StableCoin");
      const newStableCoin = await StableCoinFactory.deploy("New Stable Coin", "NSC", 1000000);
      
      await expect(factory.connect(owner).setStableCoinAddress(await newStableCoin.getAddress()))
        .to.emit(factory, "StableCoinAddressSet")
        .withArgs(await newStableCoin.getAddress());
      
      expect(await factory.stableCoin()).to.equal(await newStableCoin.getAddress());
    });
    
    it("Should only allow factory admin to change StableCoin address", async function () {
      const StableCoinFactory = await ethers.getContractFactory("StableCoin");
      const newStableCoin = await StableCoinFactory.deploy("New Stable Coin", "NSC", 1000000);
      
      await expect(factory.connect(user).setStableCoinAddress(await newStableCoin.getAddress()))
        .to.be.revertedWithCustomError(factory, "AccessControlUnauthorizedAccount")
        .withArgs(user.address, FACTORY_ADMIN_ROLE);
    });
    
    it("Should revert if new StableCoin address is zero", async function () {
      await expect(factory.connect(owner).setStableCoinAddress(ethers.ZeroAddress))
        .to.be.revertedWith("StableCoin address cannot be zero");
    });
  });
});