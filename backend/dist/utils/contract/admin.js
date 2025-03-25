"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Admin = void 0;
const ethers_1 = require("ethers");
const ethers_2 = require("ethers");
const stablecoin_1 = require("./abi/stablecoin");
const erc20factory_1 = require("./abi/erc20factory");
const tokenswap_1 = require("./abi/tokenswap");
const erc20token_1 = require("./abi/erc20token");
class Admin {
    provider;
    signer;
    address;
    stableCoin;
    tokenFactory;
    tokenSwap;
    contractCache = new Map();
    constructor(url, private_key, stableCoinAddress, tokenFactoryAddress, swapAddress) {
        this.provider = new ethers_1.ethers.JsonRpcProvider(url);
        this.signer = new ethers_1.ethers.Wallet(private_key, this.provider);
        this.stableCoin = new ethers_2.Contract(stableCoinAddress, stablecoin_1.StableCoinABI, this.signer);
        this.tokenFactory = new ethers_2.Contract(tokenFactoryAddress, erc20factory_1.ERC20FactoryABI, this.signer);
        this.tokenSwap = new ethers_2.Contract(swapAddress, tokenswap_1.TokenSwapABI, this.signer);
    }
    /**
     * Helper Methods
     */
    getContract(address, abi, useSigner = false) {
        const cacheKey = `${address}-${useSigner}`;
        if (this.contractCache.has(cacheKey)) {
            return this.contractCache.get(cacheKey);
        }
        const contract = new ethers_2.Contract(address, abi, useSigner ? this.signer : this.provider);
        this.contractCache.set(cacheKey, contract);
        return contract;
    }
    async executeTransaction(operation, errorMessage) {
        try {
            const tx = await operation();
            return (await tx.wait());
        }
        catch (error) {
            throw new Error(`${errorMessage}: ${error instanceof Error ? error.message : String(error)}`);
        }
    }
    async executeViewOperation(operation, errorMessage) {
        try {
            return await operation();
        }
        catch (error) {
            throw new Error(`${errorMessage}: ${error instanceof Error ? error.message : String(error)}`);
        }
    }
    formatTokenAmount(amount) {
        return parseFloat((0, ethers_2.formatUnits)(amount, 18));
    }
    async getSignerAddress() {
        return this.signer.getAddress();
    }
    /**
     * StableCoin Balance & Supply Methods
     */
    async checkBalance(accountAddress) {
        return this.executeViewOperation(async () => {
            const balance = await this.stableCoin.balanceOf(accountAddress);
            return this.formatTokenAmount(balance);
        }, "Failed to check balance");
    }
    async checkTotalSupply() {
        return this.executeViewOperation(async () => {
            const supply = await this.stableCoin.totalSupply();
            return this.formatTokenAmount(supply);
        }, "Failed to check Total Supply");
    }
    async checkTokenBalance(tokenAddress, accountAddress) {
        return this.executeViewOperation(async () => {
            const token = this.getContract(tokenAddress, erc20token_1.ERC20TokenABI);
            const balance = await token.balanceOf(accountAddress);
            return this.formatTokenAmount(balance);
        }, "Failed to check token balance");
    }
    async checkTokenTotalSupply(tokenAddress) {
        return this.executeViewOperation(async () => {
            const supply = await this.getContract(tokenAddress, erc20token_1.ERC20TokenABI).totalSupply();
            return this.formatTokenAmount(supply);
        }, "Failed to check token total supply");
    }
    /**
     * StableCoin Whitelist Methods
     */
    async checkWhitelist(accountAddress) {
        return this.executeViewOperation(() => this.stableCoin.whitelisted(accountAddress), "Failed to check WhiteList");
    }
    async addToWhitelist(accountAddress) {
        return this.executeTransaction(() => this.stableCoin.addToWhitelist(accountAddress), "Failed to add to WhiteList");
    }
    async removeFromWhitelist(accountAddress) {
        return this.executeTransaction(() => this.stableCoin.removeFromWhitelist(accountAddress), "Failed to remove from WhiteList");
    }
    async addBatchToWhitelist(accountAddresses) {
        return this.executeTransaction(() => this.stableCoin.batchAddToWhitelist(accountAddresses), "Failed to add batch to WhiteList");
    }
    /**
     * StableCoin Operations
     */
    async mintStableCoin(toAddress, amount) {
        return this.executeTransaction(() => this.stableCoin.mint(toAddress, (0, ethers_2.parseUnits)(amount.toString(), 18)), "Failed to mint StableCoin");
    }
    async withdrawStableCoin(amount, withdrawerAddress, reason) {
        return this.executeTransaction(() => this.stableCoin.withdraw((0, ethers_2.parseUnits)(amount.toString(), 18), withdrawerAddress, ethers_1.ethers.encodeBytes32String(reason)), "Failed to withdraw StableCoin");
    }
    async swapperStableToken(tokenAdress, amount) {
        try {
            const swapAddress = await this.tokenSwap.getAddress();
            const swapAmonut = (0, ethers_2.parseUnits)(amount.toString(), 18);
            const approve = await this.stableCoin.approve(swapAddress, swapAmonut);
            await approve.wait();
            const swapTx = await this.tokenSwap.swapStableCoinToToken(tokenAdress, swapAmonut);
            await swapTx.wait();
            return swapTx;
        }
        catch (error) {
            throw error;
        }
    }
    async swapperTokenStable(tokenAdress, amount) {
        try {
            const swapAddress = await this.tokenSwap.getAddress();
            const token = this.getContract(tokenAdress, erc20token_1.ERC20TokenABI, true);
            const ratio = await this.tokenFactory.tokenRatios(await token.getAddress());
            const swapAmonut = (0, ethers_2.parseUnits)(amount.toString(), 18); // Token Amount
            const stableCoinAmount = swapAmonut / ratio; // StableCoin Amount
            // Have the token approve the TokenSwap contract to spend its StableCoins
            const approveStable = await this.stableCoin.approve(swapAddress, stableCoinAmount);
            await approveStable.wait();
            // Approve token swap to spend tokens
            const approve = await token.approve(await this.tokenSwap.getAddress(), swapAmonut);
            await approve.wait();
            const swapTx = await this.tokenSwap.swapTokenToStableCoin(await token.getAddress(), swapAmonut);
            await swapTx.wait();
            return swapTx;
        }
        catch (error) {
            throw error;
        }
    }
    async transferStableCoin(toAddress, amount) {
        return this.executeTransaction(() => this.stableCoin.transfer(toAddress, (0, ethers_2.parseUnits)(amount.toString(), 18)), "Failed to transfer StableCoin");
    }
    /**
     * Token Factory Methods
     */
    async isTokenCreatedByFactory(tokenAddress) {
        return this.executeViewOperation(() => this.tokenFactory.isTokenCreatedByFactory(tokenAddress), "Failed to check if token created by factory");
    }
    async createToken(name, symbol, tokenOwner, tokensPerStableCoin) {
        try {
            const stableCoinAddress = await this.stableCoin.getAddress();
            const swapperAddress = await this.tokenSwap.getAddress();
            const tx = await this.tokenFactory.createToken(name, symbol, stableCoinAddress, swapperAddress, tokenOwner, tokensPerStableCoin);
            const receipt = await tx.wait();
            // Find TokenCreated event in logs
            for (const log of receipt.logs || []) {
                try {
                    const parsedLog = this.tokenFactory.interface.parseLog({
                        topics: log.topics,
                        data: log.data,
                    });
                    if (parsedLog?.name === "TokenCreated") {
                        const event = parsedLog;
                        const tokenAddress = event.args.tokenAddress;
                        const addWhiteList = await this.stableCoin.addToWhitelist(tokenAddress);
                        await addWhiteList.wait();
                        return tokenAddress;
                    }
                }
                catch (e) {
                    // Continue trying other logs if parsing fails
                    continue;
                }
            }
            throw new Error("TokenCreated event not found in transaction logs");
        }
        catch (error) {
            throw new Error(`Failed to create Token: ${error instanceof Error ? error.message : String(error)}`);
        }
    }
    async mintToken(tokenAddress, toAddress, amount) {
        try {
            const tx = await this.tokenFactory.mintToken(tokenAddress, toAddress, (0, ethers_2.parseUnits)(amount.toString(), 18));
            const receipt = await tx.wait();
            if (!receipt) {
                return { success: false, error: "Transaction receipt undefined" };
            }
            // Look for TokenMinted event
            let mintEventFound = false;
            for (const log of receipt.logs) {
                try {
                    const parsed = this.tokenFactory.interface.parseLog({
                        topics: Array.isArray(log.topics) ? log.topics : [],
                        data: log.data,
                    });
                    if (parsed?.name === "TokenMinted") {
                        mintEventFound = true;
                        break;
                    }
                }
                catch {
                    continue;
                }
            }
            return {
                success: true,
                transactionHash: receipt.hash,
                ...(!mintEventFound && {
                    blockNumber: receipt.blockNumber,
                    error: "Transaction successful but TokenMinted event not found",
                }),
            };
        }
        catch (error) {
            throw new Error(`Failed to mint Token: ${error instanceof Error ? error.message : String(error)}`);
        }
    }
    async getAllCreatedTokens() {
        return this.tokenFactory.getAllTokenAddresses();
    }
    async tokenTransfer(tokenAddress, to, amount) {
        const token = this.getContract(tokenAddress, erc20token_1.ERC20TokenABI, true);
        return this.executeTransaction(() => token.transfer(to, (0, ethers_2.parseUnits)(amount.toString(), 18)), "Failed to transfer token");
    }
}
exports.Admin = Admin;
