import { expect } from "chai";
import { ethers } from "hardhat";
import { StableCoin, ERC20Factory, TokenSwap, ERC20Token } from "../typechain";
import { SignerWithAddress } from "@nomicfoundation/hardhat-ethers/signers";

describe("TokenSwap Contract", function () {
  let stableCoin: StableCoin;
  let factory: ERC20Factory;
  let tokenSwap: TokenSwap;
  let token: ERC20Token;
  
  let owner: SignerWithAddress;
  let admin: SignerWithAddress;
  let feeCollector: SignerWithAddress;
  let user1: SignerWithAddress;
  let user2: SignerWithAddress;
  
  // Constants
  const INITIAL_FEE_PERCENTAGE = 25n; // 0.25%
  const TOKEN_RATIO = 10n; // 10 tokens per 1 stablecoin
  
  // Role constants
  const ADMIN_ROLE = ethers.keccak256(ethers.toUtf8Bytes("ADMIN_ROLE"));
  const PAUSER_ROLE = ethers.keccak256(ethers.toUtf8Bytes("PAUSER_ROLE"));
  const FEE_MANAGER_ROLE = ethers.keccak256(ethers.toUtf8Bytes("FEE_MANAGER_ROLE"));
  
  // StableCoin roles
  const MINTER_ROLE = ethers.keccak256(ethers.toUtf8Bytes("MINTER_ROLE"));
  const WHITELIST_MANAGER_ROLE = ethers.keccak256(ethers.toUtf8Bytes("WHITELIST_MANAGER_ROLE"));
  
  // Factory roles
  const TOKEN_CREATOR_ROLE = ethers.keccak256(ethers.toUtf8Bytes("TOKEN_CREATOR_ROLE"));
  const FACTORY_MINTER_ROLE = ethers.keccak256(ethers.toUtf8Bytes("FACTORY_MINTER_ROLE"));
  
  const DECIMALS = 18n;  // Assuming both tokens have 18 decimals
  const BASIS_POINTS = 10000n;
  
  beforeEach(async function () {
    [owner, admin, feeCollector, user1, user2] = await ethers.getSigners();
    
    // Deploy StableCoin
    const StableCoinFactory = await ethers.getContractFactory("StableCoin");
    stableCoin = await StableCoinFactory.deploy("Test Stable Coin", "TSC", 1000000);
    
    // Set up StableCoin
    await stableCoin.grantRole(MINTER_ROLE, owner.address);
    await stableCoin.grantRole(WHITELIST_MANAGER_ROLE, owner.address);
    
    // Whitelist all relevant addresses
    await stableCoin.addToWhitelist(owner.address);
    await stableCoin.addToWhitelist(user1.address);
    await stableCoin.addToWhitelist(user2.address);
    await stableCoin.addToWhitelist(feeCollector.address);
    
    // Deploy Factory
    const ERC20FactoryContract = await ethers.getContractFactory("ERC20Factory");
    factory = await ERC20FactoryContract.deploy(owner.address, await stableCoin.getAddress());
    
    // Set up Factory roles
    await factory.grantRole(TOKEN_CREATOR_ROLE, owner.address);
    await factory.grantRole(FACTORY_MINTER_ROLE, owner.address);
    
    // Create a token through the factory
    const tx = await factory.createToken(
      "Test Token",
      "TT",
      owner.address,
      TOKEN_RATIO
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
    
    const tokenAddress = event.args[1];
    token = await ethers.getContractAt("ERC20Token", tokenAddress);
    
    // Deploy TokenSwap
    const TokenSwapFactory = await ethers.getContractFactory("TokenSwap");
    tokenSwap = await TokenSwapFactory.deploy(
      await stableCoin.getAddress(),
      await factory.getAddress(),
      admin.address,
      feeCollector.address,
      INITIAL_FEE_PERCENTAGE
    );
    
    // Whitelist the TokenSwap contract address
    await stableCoin.addToWhitelist(await tokenSwap.getAddress());
    
    // *** IMPORTANT: Fund the TokenSwap contract with StableCoin ***
    // This is required for the TokenSwap to have StableCoin to give during Token->StableCoin swaps
    const swapFundAmount = 100000n * (10n ** DECIMALS);
    await stableCoin.mint(await tokenSwap.getAddress(), swapFundAmount);
    
    // Mint tokens for testing
    const tokenAmount = 1000000n * (10n ** DECIMALS);
    
    // Mint tokens to users
    await factory.mintToken(tokenAddress, owner.address, tokenAmount);
    await factory.mintToken(tokenAddress, user2.address, tokenAmount / 10n);
    
    // Transfer tokens to the TokenSwap contract (for StableCoin->Token swaps)
    await token.transfer(await tokenSwap.getAddress(), tokenAmount / 2n);
    
    // Give StableCoin to users for testing
    const stableCoinAmount = 10000n * (10n ** DECIMALS);
    await stableCoin.mint(user1.address, stableCoinAmount);
    
    // Setup approvals
    await stableCoin.connect(user1).approve(await tokenSwap.getAddress(), stableCoinAmount);
    await token.connect(user2).approve(await tokenSwap.getAddress(), tokenAmount / 10n);
  });
  
  describe("Deployment", function () {
    it("Should set the correct stablecoin and factory addresses", async function () {
      expect(await tokenSwap.stableCoin()).to.equal(await stableCoin.getAddress());
      expect(await tokenSwap.tokenFactory()).to.equal(await factory.getAddress());
    });
    
    it("Should set the correct fee percentage and collector", async function () {
      expect(await tokenSwap.feePercentage()).to.equal(INITIAL_FEE_PERCENTAGE);
      expect(await tokenSwap.feeCollector()).to.equal(feeCollector.address);
    });
    
    it("Should set the correct roles to admin", async function () {
      expect(await tokenSwap.hasRole(ethers.ZeroHash, admin.address)).to.be.true; // DEFAULT_ADMIN_ROLE
      expect(await tokenSwap.hasRole(ADMIN_ROLE, admin.address)).to.be.true;
      expect(await tokenSwap.hasRole(PAUSER_ROLE, admin.address)).to.be.true;
      expect(await tokenSwap.hasRole(FEE_MANAGER_ROLE, admin.address)).to.be.true;
    });
    
    it("Should revert if invalid parameters are provided", async function () {
      const TokenSwapFactory = await ethers.getContractFactory("TokenSwap");
      
      await expect(TokenSwapFactory.deploy(
        ethers.ZeroAddress,
        await factory.getAddress(),
        admin.address,
        feeCollector.address,
        INITIAL_FEE_PERCENTAGE
      )).to.be.revertedWith("StableCoin cannot be zero address");
      
      await expect(TokenSwapFactory.deploy(
        await stableCoin.getAddress(),
        ethers.ZeroAddress,
        admin.address,
        feeCollector.address,
        INITIAL_FEE_PERCENTAGE
      )).to.be.revertedWith("TokenFactory cannot be zero address");
      
      await expect(TokenSwapFactory.deploy(
        await stableCoin.getAddress(),
        await factory.getAddress(),
        ethers.ZeroAddress,
        feeCollector.address,
        INITIAL_FEE_PERCENTAGE
      )).to.be.revertedWith("Admin cannot be zero address");
      
      await expect(TokenSwapFactory.deploy(
        await stableCoin.getAddress(),
        await factory.getAddress(),
        admin.address,
        ethers.ZeroAddress,
        INITIAL_FEE_PERCENTAGE
      )).to.be.revertedWith("Fee collector cannot be zero address");
      
      const MAX_FEE = await tokenSwap.MAX_FEE();
      await expect(TokenSwapFactory.deploy(
        await stableCoin.getAddress(),
        await factory.getAddress(),
        admin.address,
        feeCollector.address,
        MAX_FEE + 1n
      )).to.be.revertedWith("Fee too high");
    });
  });
  
  describe("Swap StableCoin to Token", function () {
    it("Should swap stablecoin for tokens at the correct ratio", async function () {
      const stableCoinAmount = 100n * (10n ** DECIMALS);
      const expectedTokenAmount = stableCoinAmount * TOKEN_RATIO;
      
      // Get initial balances
      const initialStableCoinBalance = await stableCoin.balanceOf(user1.address);
      const initialTokenBalance = await token.balanceOf(user1.address);
      const initialSwapStableCoinBalance = await stableCoin.balanceOf(await tokenSwap.getAddress());
      const initialFeeCollectorBalance = await stableCoin.balanceOf(feeCollector.address);
      
      // Calculate expected fee
      const feeAmount = (stableCoinAmount * INITIAL_FEE_PERCENTAGE) / BASIS_POINTS;
      
      // Perform the swap
      await expect(tokenSwap.connect(user1).swapStableCoinToToken(
        await token.getAddress(),
        stableCoinAmount
      ))
        .to.emit(tokenSwap, "StableCoinToToken")
        .withArgs(user1.address, await token.getAddress(), stableCoinAmount, expectedTokenAmount, feeAmount);
      
      // Check balances after swap
      expect(await stableCoin.balanceOf(user1.address)).to.equal(initialStableCoinBalance - stableCoinAmount);
      expect(await token.balanceOf(user1.address)).to.equal(initialTokenBalance + expectedTokenAmount);
      expect(await stableCoin.balanceOf(await tokenSwap.getAddress())).to.equal(
        initialSwapStableCoinBalance + stableCoinAmount - feeAmount
      );
      expect(await stableCoin.balanceOf(feeCollector.address)).to.equal(
        initialFeeCollectorBalance + feeAmount
      );
    });
    
    it("Should revert if user is not whitelisted", async function () {
      const nonWhitelistedUser = (await ethers.getSigners())[5];
      const stableCoinAmount = 100n * (10n ** DECIMALS);
      
      // Mint and approve
      await stableCoin.mint(nonWhitelistedUser.address, stableCoinAmount);
      await stableCoin.connect(nonWhitelistedUser).approve(await tokenSwap.getAddress(), stableCoinAmount);
      
      await expect(tokenSwap.connect(nonWhitelistedUser).swapStableCoinToToken(
        await token.getAddress(),
        stableCoinAmount
      )).to.be.revertedWith("User not whitelisted");
    });
    
    it("Should revert if token is not supported by factory", async function () {
      // Deploy a direct token not from factory
      const ERC20TokenFactory = await ethers.getContractFactory("ERC20Token");
      const unsupportedToken = await ERC20TokenFactory.deploy(
        "Unsupported Token",
        "UNSUP",
        owner.address
      );
      
      const stableCoinAmount = 100n * (10n ** DECIMALS);
      
      await expect(tokenSwap.connect(user1).swapStableCoinToToken(
        await unsupportedToken.getAddress(),
        stableCoinAmount
      )).to.be.revertedWith("Token not supported");
    });
    
    it("Should revert if amount is zero", async function () {
      await expect(tokenSwap.connect(user1).swapStableCoinToToken(
        await token.getAddress(),
        0
      )).to.be.revertedWith("Amount must be greater than zero");
    });
  });
  
  describe("Swap Token to StableCoin", function () {
    it("Should swap tokens for stablecoin at the correct ratio", async function () {
      const tokenAmount = 100n * (10n ** DECIMALS);
      const expectedStableCoinAmount = tokenAmount / TOKEN_RATIO;
      
      // Get initial balances
      const initialTokenBalance = await token.balanceOf(user2.address);
      const initialStableCoinBalance = await stableCoin.balanceOf(user2.address);
      const initialSwapTokenBalance = await token.balanceOf(await tokenSwap.getAddress());
      const initialSwapStableCoinBalance = await stableCoin.balanceOf(await tokenSwap.getAddress());
      const initialFeeCollectorBalance = await stableCoin.balanceOf(feeCollector.address);
      
      // Calculate expected fee
      const feeAmount = (expectedStableCoinAmount * INITIAL_FEE_PERCENTAGE) / BASIS_POINTS;
      const expectedTransferAmount = expectedStableCoinAmount - feeAmount;
      
      // Perform the swap
      await expect(tokenSwap.connect(user2).swapTokenToStableCoin(
        await token.getAddress(),
        tokenAmount
      ))
        .to.emit(tokenSwap, "TokenToStableCoin")
        .withArgs(user2.address, await token.getAddress(), tokenAmount, expectedStableCoinAmount, feeAmount);
      
      // Check balances after swap
      expect(await token.balanceOf(user2.address)).to.equal(initialTokenBalance - tokenAmount);
      expect(await stableCoin.balanceOf(user2.address)).to.equal(initialStableCoinBalance + expectedTransferAmount);
      expect(await token.balanceOf(await tokenSwap.getAddress())).to.equal(
        initialSwapTokenBalance + tokenAmount
      );
      // With a fee percentage of 0.25% (25 basis points), the fee is 0.025 StableCoins
      expect(await stableCoin.balanceOf(await tokenSwap.getAddress())).to.equal(
        initialSwapStableCoinBalance - expectedTransferAmount - BigInt("25000000000000000")
      );
      expect(await stableCoin.balanceOf(feeCollector.address)).to.equal(
        initialFeeCollectorBalance + feeAmount
      );
    });
    
    it("Should revert if user is not whitelisted", async function () {
      const nonWhitelistedUser = (await ethers.getSigners())[5];
      const tokenAmount = 100n * (10n ** DECIMALS);
      
      // Transfer tokens and approve
      await token.connect(user2).transfer(nonWhitelistedUser.address, tokenAmount);
      await token.connect(nonWhitelistedUser).approve(await tokenSwap.getAddress(), tokenAmount);
      
      await expect(tokenSwap.connect(nonWhitelistedUser).swapTokenToStableCoin(
        await token.getAddress(),
        tokenAmount
      )).to.be.revertedWith("User not whitelisted");
    });
    
    it("Should revert if token is not supported by factory", async function () {
      // Deploy a direct token not from factory
      const ERC20TokenFactory = await ethers.getContractFactory("ERC20Token");
      const unsupportedToken = await ERC20TokenFactory.deploy(
        "Unsupported Token",
        "UNSUP",
        owner.address
      );
      
      const tokenAmount = 100n * (10n ** DECIMALS);
      
      await expect(tokenSwap.connect(user2).swapTokenToStableCoin(
        await unsupportedToken.getAddress(),
        tokenAmount
      )).to.be.revertedWith("Token not supported");
    });
    
    it("Should revert if amount is zero", async function () {
      await expect(tokenSwap.connect(user2).swapTokenToStableCoin(
        await token.getAddress(),
        0
      )).to.be.revertedWith("Amount must be greater than zero");
    });
    
    it("Should revert if resulting StableCoin amount is zero due to small token amount", async function () {
      // Try with an amount lower than the token ratio (which would result in 0 stablecoin)
      const smallAmount = TOKEN_RATIO - 1n;
      
      await expect(tokenSwap.connect(user2).swapTokenToStableCoin(
        await token.getAddress(),
        smallAmount
      )).to.be.revertedWith("StableCoin amount too small");
    });
  });
  
  describe("Fee Management", function () {
    it("Should allow fee manager to update fee percentage", async function () {
      const newFeePercentage = 50n; // 0.5%
      
      await expect(tokenSwap.connect(admin).setFeePercentage(newFeePercentage))
        .to.emit(tokenSwap, "FeeUpdated")
        .withArgs(newFeePercentage);
      
      expect(await tokenSwap.feePercentage()).to.equal(newFeePercentage);
    });
    
    it("Should revert if fee is set too high", async function () {
      const MAX_FEE = await tokenSwap.MAX_FEE();
      
      await expect(tokenSwap.connect(admin).setFeePercentage(MAX_FEE + 1n))
        .to.be.revertedWith("Fee too high");
    });
    
    it("Should only allow fee manager to update fee", async function () {
      await expect(tokenSwap.connect(user1).setFeePercentage(50))
        .to.be.revertedWithCustomError(tokenSwap, "AccessControlUnauthorizedAccount")
        .withArgs(user1.address, FEE_MANAGER_ROLE);
    });
    
    it("Should allow admin to update fee collector", async function () {
      const newFeeCollector = user2.address;
      
      await expect(tokenSwap.connect(admin).setFeeCollector(newFeeCollector))
        .to.emit(tokenSwap, "FeeCollectorUpdated")
        .withArgs(newFeeCollector);
      
      expect(await tokenSwap.feeCollector()).to.equal(newFeeCollector);
    });
    
    it("Should revert if fee collector is zero address", async function () {
      await expect(tokenSwap.connect(admin).setFeeCollector(ethers.ZeroAddress))
        .to.be.revertedWith("Fee collector cannot be zero address");
    });
  });
  
  describe("Pause Functionality", function () {
    it("Should pause swaps when paused", async function () {
      await tokenSwap.connect(admin).pause();
      
      const stableCoinAmount = 100n * (10n ** DECIMALS);
      const tokenAmount = 100n * (10n ** DECIMALS);
      
      await expect(tokenSwap.connect(user1).swapStableCoinToToken(
        await token.getAddress(),
        stableCoinAmount
      )).to.be.reverted;
      
      await expect(tokenSwap.connect(user2).swapTokenToStableCoin(
        await token.getAddress(),
        tokenAmount
      )).to.be.reverted;
    });
    
    it("Should resume swaps when unpaused", async function () {
      await tokenSwap.connect(admin).pause();
      await tokenSwap.connect(admin).unpause();
      
      const stableCoinAmount = 100n * (10n ** DECIMALS);
      
      await expect(tokenSwap.connect(user1).swapStableCoinToToken(
        await token.getAddress(),
        stableCoinAmount
      )).to.not.be.reverted;
    });
    
    it("Should only allow pausers to pause/unpause", async function () {
      await expect(tokenSwap.connect(user1).pause())
        .to.be.revertedWithCustomError(tokenSwap, "AccessControlUnauthorizedAccount")
        .withArgs(user1.address, PAUSER_ROLE);
      
      await tokenSwap.connect(admin).pause();
      
      await expect(tokenSwap.connect(user1).unpause())
        .to.be.revertedWithCustomError(tokenSwap, "AccessControlUnauthorizedAccount")
        .withArgs(user1.address, PAUSER_ROLE);
    });
  });
  
  describe("Emergency Withdraw", function () {
    it("Should allow admin to withdraw tokens in emergency", async function () {
      const withdrawAmount = 500n * (10n ** DECIMALS);
      const recipient = user2.address;
      
      const initialBalance = await token.balanceOf(recipient);
      
      await expect(tokenSwap.connect(admin).emergencyWithdraw(
        await token.getAddress(),
        withdrawAmount,
        recipient
      )).to.not.be.reverted;
      
      expect(await token.balanceOf(recipient)).to.equal(initialBalance + withdrawAmount);
    });
    
    it("Should revert if non-admin tries to emergency withdraw", async function () {
      await expect(tokenSwap.connect(user1).emergencyWithdraw(
        await token.getAddress(),
        100,
        user2.address
      ))
        .to.be.revertedWithCustomError(tokenSwap, "AccessControlUnauthorizedAccount")
        .withArgs(user1.address, ADMIN_ROLE);
    });
    
    it("Should revert if trying to withdraw to zero address", async function () {
      await expect(tokenSwap.connect(admin).emergencyWithdraw(
        await token.getAddress(),
        100,
        ethers.ZeroAddress
      )).to.be.revertedWith("Cannot withdraw to zero address");
    });
  });
  
  describe("Integration Tests", function () {
    it("Should handle complete swap cycle (StableCoin -> Token -> StableCoin)", async function () {
      // First give some StableCoin to user1 for testing
      const stableCoinForCycle = 1000n * (10n ** DECIMALS);
      await stableCoin.mint(user1.address, stableCoinForCycle);
      await stableCoin.connect(user1).approve(await tokenSwap.getAddress(), stableCoinForCycle);
      
      // Get initial balance
      const initialStableCoinBalance = await stableCoin.balanceOf(user1.address);
      
      // 1. User1 swaps StableCoin for Token
      const stableCoinAmount = 100n * (10n ** DECIMALS);
      await tokenSwap.connect(user1).swapStableCoinToToken(
        await token.getAddress(),
        stableCoinAmount
      );
      
      const tokenBalanceAfterSwap = await token.balanceOf(user1.address);
      
      // 2. User1 approves TokenSwap to spend the tokens
      await token.connect(user1).approve(await tokenSwap.getAddress(), tokenBalanceAfterSwap);
      
      // 3. User1 swaps Token back to StableCoin
      await tokenSwap.connect(user1).swapTokenToStableCoin(
        await token.getAddress(),
        tokenBalanceAfterSwap
      );
      
      // 4. Verify the final StableCoin balance is less than initial due to fees
      const finalStableCoinBalance = await stableCoin.balanceOf(user1.address);
      expect(finalStableCoinBalance).to.be.lessThan(initialStableCoinBalance);
      
      // 5. Verify the fee collector received fees
      expect(await stableCoin.balanceOf(feeCollector.address)).to.be.greaterThan(0);
    });
  });
});