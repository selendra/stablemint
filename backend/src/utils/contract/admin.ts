import { ethers } from "ethers";
import {
  Contract,
  Provider,
  Signer,
  ContractTransactionResponse,
  formatUnits,
  parseUnits,
  keccak256,
  toUtf8Bytes,
} from "ethers";
import {
  ERC20FACTORY_ADMIN_ABI,
  PAUSE_ADMIN_ABI,
  ROLE_ADMIN_ABI,
  STABLECOIN_ADMIN_ABI,
  TOKENSWAP_ADMIN_ABI,
} from "./abi";

interface TokenCreatedEvent {
  args: {
    creator: string;
    tokenAddress: string;
    name: string;
    symbol: string;
    owner: string;
  };
}

export class Admin {
  private provider: ethers.JsonRpcProvider;
  private signer: Signer;
  public address?: string;
  public stableCoin: ethers.Contract;
  public tokenFactory: ethers.Contract;
  public tokenSwap: ethers.Contract;
  private contractCache: Map<string, Contract> = new Map();

  constructor(
    url: string,
    private_key: string,
    stableCoinAddress: string,
    tokenFactoryAddress: string,
    swapAddress: string
  ) {
    this.provider = new ethers.JsonRpcProvider(url);
    this.signer = new ethers.Wallet(private_key, this.provider);
    this.stableCoin = new Contract(
      stableCoinAddress,
      STABLECOIN_ADMIN_ABI,
      this.signer
    );
    this.tokenFactory = new Contract(
      tokenFactoryAddress,
      ERC20FACTORY_ADMIN_ABI,
      this.signer
    );
    this.tokenSwap = new Contract(
      swapAddress,
      TOKENSWAP_ADMIN_ABI,
      this.signer
    );
  }

  /**
   * Helper Methods
   */
  private getContract(address: string, abi: any, useSigner = false): Contract {
    const cacheKey = `${address}-${useSigner}`;

    if (this.contractCache.has(cacheKey)) {
      return this.contractCache.get(cacheKey)!;
    }

    const contract = new Contract(
      address,
      abi,
      useSigner ? this.signer : this.provider
    );

    this.contractCache.set(cacheKey, contract);
    return contract;
  }

  private async executeTransaction<T>(
    operation: () => Promise<ContractTransactionResponse>,
    errorMessage: string
  ): Promise<T> {
    try {
      const tx = await operation();
      return (await tx.wait()) as unknown as T;
    } catch (error) {
      throw new Error(
        `${errorMessage}: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    }
  }

  private async executeViewOperation<T>(
    operation: () => Promise<T>,
    errorMessage: string
  ): Promise<T> {
    try {
      return await operation();
    } catch (error) {
      throw new Error(
        `${errorMessage}: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    }
  }

  private formatTokenAmount(amount: bigint): number {
    return parseFloat(formatUnits(amount, 18));
  }

  async getSignerAddress(): Promise<string> {
    return this.signer.getAddress();
  }

  /**
   * StableCoin Balance & Supply Methods
   */
  async checkBalance(accountAddress: string): Promise<number> {
    return this.executeViewOperation(async () => {
      const balance = await this.stableCoin.balanceOf(accountAddress);
      return this.formatTokenAmount(balance);
    }, "Failed to check balance");
  }

  async checkTotalSupply(): Promise<number> {
    return this.executeViewOperation(async () => {
      const supply = await this.stableCoin.totalSupply();
      return this.formatTokenAmount(supply);
    }, "Failed to check Total Supply");
  }

  async checkTokenBalance(
    tokenAddress: string,
    accountAddress: string
  ): Promise<number> {
    return this.executeViewOperation(async () => {
      const token = this.getContract(tokenAddress, ERC20FACTORY_ADMIN_ABI);
      const balance = await token.balanceOf(accountAddress);
      return this.formatTokenAmount(balance);
    }, "Failed to check token balance");
  }

  async checkTokenTotalSupply(tokenAddress: string): Promise<number> {
    return this.executeViewOperation(async () => {
      const token = this.getContract(tokenAddress, ERC20FACTORY_ADMIN_ABI);
      const supply = await token.totalSupply();
      return this.formatTokenAmount(supply);
    }, "Failed to check token total supply");
  }

  /**
   * StableCoin Whitelist Methods
   */
  async checkEnforceWhitelist(): Promise<boolean> {
    return this.executeViewOperation(
      () => this.stableCoin.enforceWhitelistForReceivers(),
      "Failed to check enforceWhitelistForReceivers"
    );
  }

  async checkWhitelist(accountAddress: string): Promise<boolean> {
    return this.executeViewOperation(
      () => this.stableCoin.whitelisted(accountAddress),
      "Failed to check WhiteList"
    );
  }

  async setWhitelistReceiverPolicy(enforceForReceivers: boolean) {
    return this.executeTransaction(
      () => this.stableCoin.setWhitelistReceiverPolicy(enforceForReceivers),
      "Failed to set WhiteList Policy"
    );
  }

  async addToWhitelist(accountAddress: string) {
    return this.executeTransaction(
      () => this.stableCoin.addToWhitelist(accountAddress),
      "Failed to add to WhiteList"
    );
  }

  async removeFromWhitelist(accountAddress: string) {
    return this.executeTransaction(
      () => this.stableCoin.removeFromWhitelist(accountAddress),
      "Failed to remove from WhiteList"
    );
  }

  async addBatchToWhitelist(accountAddresses: string[]) {
    return this.executeTransaction(
      () => this.stableCoin.batchAddToWhitelist(accountAddresses),
      "Failed to add batch to WhiteList"
    );
  }

  /**
   * StableCoin Operations
   */
  async mintStableCoin(toAddress: string, amount: number) {
    return this.executeTransaction(
      () => this.stableCoin.mint(toAddress, parseUnits(amount.toString(), 18)),
      "Failed to mint StableCoin"
    );
  }

  async withdrawStableCoin(
    amount: number,
    withdrawerAddress: string,
    reason: string
  ) {
    return this.executeTransaction(
      () =>
        this.stableCoin.withdraw(
          parseUnits(amount.toString(), 18),
          withdrawerAddress,
          ethers.encodeBytes32String(reason)
        ),
      "Failed to withdraw StableCoin"
    );
  }

  async transferStableCoin(toAddress: string, amount: number) {
    return this.executeTransaction(
      () =>
        this.stableCoin.transfer(toAddress, parseUnits(amount.toString(), 18)),
      "Failed to transfer StableCoin"
    );
  }

  /**
   * Token Factory Methods
   */
  async isTokenCreatedByFactory(tokenAddress: string): Promise<boolean> {
    return this.executeViewOperation(
      () => this.tokenFactory.isTokenCreatedByFactory(tokenAddress),
      "Failed to check if token created by factory"
    );
  }

  async createToken(
    name: string,
    symbol: string,
    tokenOwner: string,
    tokensPerStableCoin: number
  ): Promise<string> {
    try {
      const tx = await this.tokenFactory.createToken(
        name,
        symbol,
        tokenOwner,
        tokensPerStableCoin
      );
      const receipt = await tx.wait();

      // Find TokenCreated event in logs
      for (const log of receipt.logs || []) {
        try {
          const parsedLog = this.tokenFactory.interface.parseLog({
            topics: log.topics as string[],
            data: log.data,
          });

          if (parsedLog?.name === "TokenCreated") {
            const event = parsedLog as unknown as TokenCreatedEvent;
            return event.args.tokenAddress;
          }
        } catch (e) {
          // Continue trying other logs if parsing fails
          continue;
        }
      }

      throw new Error("TokenCreated event not found in transaction logs");
    } catch (error) {
      throw new Error(
        `Failed to create Token: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    }
  }

  async mintToken(tokenAddress: string, toAddress: string, amount: number) {
    try {
      const tx = await this.tokenFactory.mintToken(
        tokenAddress,
        toAddress,
        parseUnits(amount.toString(), 18)
      );
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
        } catch {
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
    } catch (error) {
      throw new Error(
        `Failed to mint Token: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    }
  }

  async getAllCreatedTokens(): Promise<string[]> {
    return this.tokenFactory.getAllTokenAddresses();
  }

  /**
   * Role Management
   */
  async grantRole(
    contractAddress: string,
    role: string,
    accountAddress: string
  ) {
    const contract = this.getContract(contractAddress, ROLE_ADMIN_ABI, true);
    const roleHash = keccak256(toUtf8Bytes(role));

    return this.executeTransaction(
      () => contract.grantRole(roleHash, accountAddress),
      "Failed to Grant Role"
    );
  }

  async revokeRole(
    contractAddress: string,
    role: string,
    accountAddress: string
  ) {
    const contract = this.getContract(contractAddress, ROLE_ADMIN_ABI, true);
    const roleHash = keccak256(toUtf8Bytes(role));

    return this.executeTransaction(
      () => contract.revokeRole(roleHash, accountAddress),
      "Failed to Revoke Role"
    );
  }

  async hasRole(
    contractAddress: string,
    role: string,
    accountAddress: string
  ): Promise<boolean> {
    return this.executeViewOperation(async () => {
      const contract = new Contract(
        contractAddress,
        ROLE_ADMIN_ABI,
        this.provider
      );
      const roleHash = keccak256(toUtf8Bytes(role));
      return await contract.hasRole(roleHash, accountAddress);
    }, "Failed to Get Role");
  }

  /**
   * Pause/Unpause Functionality
   */
  async isPausedContract(contractAddress: string): Promise<boolean> {
    const contract = this.getContract(contractAddress, PAUSE_ADMIN_ABI);

    return this.executeViewOperation(
      () => contract.paused(),
      "Failed to check pause status"
    );
  }

  async pauseContract(contractAddress: string) {
    const contract = this.getContract(contractAddress, PAUSE_ADMIN_ABI, true);

    return this.executeTransaction(
      () => contract.pause(),
      "Failed to pause contract"
    );
  }

  async unpauseContract(contractAddress: string) {
    const contract = this.getContract(contractAddress, PAUSE_ADMIN_ABI, true);

    return this.executeTransaction(
      () => contract.unpause(),
      "Failed to unpause contract"
    );
  }

  /**
   * Token Swap Methods
   */
  async setFeePercentage(feePercentage: number) {
    return this.executeTransaction(
      () => this.tokenSwap.setFeePercentage(feePercentage),
      "Failed to set Fee Percentage"
    );
  }

  async setFeeCollector(feeCollectorAddress: string) {
    return this.executeTransaction(
      () => this.tokenSwap.setFeeCollector(feeCollectorAddress),
      "Failed to set Fee Collector"
    );
  }

  async emergencyWithdraw(
    tokenAddress: string,
    amount: number,
    toAddress: string
  ) {
    return this.executeTransaction(
      () =>
        this.tokenSwap.emergencyWithdraw(
          tokenAddress,
          parseUnits(amount.toString(), 18),
          toAddress
        ),
      "Failed to emergency withdraw"
    );
  }
}
