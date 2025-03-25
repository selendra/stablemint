// user-interactions.ts
// Class for regular users to interact with the smart contract system
// Updated for ethers v6.13.5

import { ethers } from "ethers";
import {
  Contract,
  Provider,
  BrowserProvider,
  Signer,
  ContractTransactionResponse,
  TransactionReceipt,
  formatUnits,
  parseUnits,
} from "ethers";
import {
  ERC20FACTORY_ABI,
  ERC20TOKEN_ABI,
  STABLECOIN_ABI,
  TOKENSWAP_ABI,
} from "./abi";

export type ConnectWalletResult = {
  signer: Signer;
  address: string;
};

export class User {
  private signer?: Signer;
  public address?: string;
  private provider: Provider;
  private contractCache: Map<string, Contract> = new Map();
  public readonly stableCoinAddress: string;
  public readonly swapAddress: string;
  public readonly factoryAddress: string;

  constructor(
    provider: Provider,
    stableCoinAddress: string,
    swapAddress: string,
    tokenFactoryAddress: string
  ) {
    this.provider = provider;
    this.swapAddress = stableCoinAddress;
    this.stableCoinAddress = swapAddress;
    this.factoryAddress = tokenFactoryAddress;
  }

  /**
   * Helper method to get a contract instance (with caching)
   */
  private getContract(address: string, abi: any, useSigner = false): Contract {
    const cacheKey = `${address}-${useSigner}`;

    if (this.contractCache.has(cacheKey)) {
      return this.contractCache.get(cacheKey)!;
    }

    const contract = new Contract(
      address,
      abi,
      useSigner && this.signer ? this.signer : this.provider
    );

    this.contractCache.set(cacheKey, contract);
    return contract;
  }

  /**
   * Execute a transaction with proper error handling
   */
  private async executeTransaction<T>(
    operation: () => Promise<ContractTransactionResponse>,
    errorMessage: string
  ): Promise<TransactionReceipt | null> {
    try {
      const tx = await operation();
      return await tx.wait();
    } catch (error) {
      throw new Error(
        `${errorMessage}: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    }
  }

  /**
   * Execute a view operation with proper error handling
   */
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

  /**
   * Connect to wallet and set signer
   */
  async connectWallet(browserProvider: any): Promise<ConnectWalletResult> {
    try {
      const provider = new BrowserProvider(browserProvider);
      await provider.send("eth_requestAccounts", []);
      this.signer = await provider.getSigner();
      this.address = await this.signer.getAddress();

      // Clear contract cache when wallet changes
      this.contractCache.clear();

      return { signer: this.signer, address: this.address };
    } catch (error) {
      throw new Error(
        `Failed to connect wallet: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    }
  }

  /**
   * Check if connected
   */
  isConnected(): boolean {
    return !!this.signer && !!this.address;
  }

  /**
   * Get connected address
   */
  getAddress(): string {
    if (!this.address) {
      throw new Error("Wallet not connected");
    }
    return this.address;
  }

  /**
   * Disconnect wallet
   */
  disconnect(): void {
    this.signer = undefined;
    this.address = undefined;
    this.contractCache.clear();
  }

  // ======== StableCoin Functions ========

  /**
   * Check if user is whitelisted
   */
  async checkWhitelistStatus(userAddress?: string): Promise<boolean> {
    const address = userAddress || this.address;
    if (!address) throw new Error("No address provided");

    const contract = this.getContract(this.stableCoinAddress, STABLECOIN_ABI);
    return this.executeViewOperation(
      () => contract.whitelisted(address),
      "Failed to check whitelist status"
    );
  }

  /**
   * Transfer stable coins to another address
   */
  async transferStableCoin(
    toAddress: string,
    amount: number
  ): Promise<TransactionReceipt | null> {
    if (!this.signer) throw new Error("Wallet not connected");

    const contract = this.getContract(
      this.stableCoinAddress,
      STABLECOIN_ABI,
      true
    );
    return this.executeTransaction(
      () => contract.transfer(toAddress, parseUnits(amount.toString(), 18)),
      "Failed to transfer stable coin"
    );
  }

  /**
   * Approve spender to use stable coins
   */
  async approveStableCoin(
    spenderAddress: string,
    amount: number
  ): Promise<TransactionReceipt | null> {
    if (!this.signer) throw new Error("Wallet not connected");

    const contract = this.getContract(
      this.stableCoinAddress,
      STABLECOIN_ABI,
      true
    );
    return this.executeTransaction(
      () => contract.approve(spenderAddress, parseUnits(amount.toString(), 18)),
      "Failed to approve stable coin"
    );
  }

  /**
   * Get stable coin balance
   */
  async getStableCoinBalance(userAddress?: string): Promise<string> {
    const address = userAddress || this.address;
    if (!address) throw new Error("No address provided");

    const contract = this.getContract(this.stableCoinAddress, STABLECOIN_ABI);
    return this.executeViewOperation(async () => {
      const balance = await contract.balanceOf(address);
      return formatUnits(balance, 18);
    }, "Failed to get stable coin balance");
  }
  // ======== TokenSwap Functions ========

  /**
   * Swap stable coins to tokens
   */
  async swapStableCoinToToken(
    tokenAddress: string,
    stableCoinAmount: number
  ): Promise<TransactionReceipt | null> {
    if (!this.signer) throw new Error("Wallet not connected");

    const contract = this.getContract(this.swapAddress, TOKENSWAP_ABI, true);
    return this.executeTransaction(
      () =>
        contract.swapStableCoinToToken(
          tokenAddress,
          parseUnits(stableCoinAmount.toString(), 18)
        ),
      "Failed to swap stable coin to token"
    );
  }

  /**
   * Swap tokens to stable coins
   */
  async swapTokenToStableCoin(
    tokenAddress: string,
    tokenAmount: number
  ): Promise<TransactionReceipt | null> {
    if (!this.signer) throw new Error("Wallet not connected");

    const contract = this.getContract(this.swapAddress, TOKENSWAP_ABI, true);
    return this.executeTransaction(
      () =>
        contract.swapTokenToStableCoin(
          tokenAddress,
          parseUnits(tokenAmount.toString(), 18)
        ),
      "Failed to swap token to stable coin"
    );
  }

  // ======== System Information Functions ========

  /**
   * Get token to stable coin ratio
   */
  async getTokenRatio(tokenAddress: string): Promise<string> {
    const contract = this.getContract(this.factoryAddress, ERC20FACTORY_ABI);
    return this.executeViewOperation(async () => {
      const ratio = await contract.tokenRatios(tokenAddress);
      return ratio.toString();
    }, "Failed to get token ratio");
  }

  /**
   * Get swap fee percentage
   */
  async getFeePercentage(): Promise<string> {
    const contract = this.getContract(this.swapAddress, TOKENSWAP_ABI);
    return this.executeViewOperation(async () => {
      const fee = await contract.feePercentage();
      return fee.toString();
    }, "Failed to get fee percentage");
  }

  /**
   * Check if system is paused
   */
  async isSystemPaused(): Promise<boolean> {
    const contract = this.getContract(this.stableCoinAddress, STABLECOIN_ABI);
    return this.executeViewOperation(
      () => contract.paused(),
      "Failed to check if system is paused"
    );
  }

  /**
   * Get token name
   */
  async getTokenName(tokenAddress: string): Promise<string> {
    const contract = this.getContract(tokenAddress, ERC20TOKEN_ABI);
    return this.executeViewOperation(
      () => contract.name(),
      "Failed to get token name"
    );
  }

  /**
   * Get token symbol
   */
  async getTokenSymbol(tokenAddress: string): Promise<string> {
    const contract = this.getContract(tokenAddress, ERC20TOKEN_ABI);
    return this.executeViewOperation(
      () => contract.symbol(),
      "Failed to get token symbol"
    );
  }

  /**
   * Get token decimals
   */
  async getTokenDecimals(tokenAddress: string): Promise<number> {
    const contract = this.getContract(tokenAddress, ERC20TOKEN_ABI);
    return this.executeViewOperation(
      () => contract.decimals(),
      "Failed to get token decimals"
    );
  }

  // ======== ERC20Token Functions ========

  /**
   * Transfer tokens to another address
   */
  async transferToken(
    tokenAddress: string,
    toAddress: string,
    amount: number
  ): Promise<TransactionReceipt | null> {
    if (!this.signer) throw new Error("Wallet not connected");

    const contract = this.getContract(tokenAddress, ERC20TOKEN_ABI, true);
    return this.executeTransaction(
      () => contract.transfer(toAddress, parseUnits(amount.toString(), 18)),
      "Failed to transfer token"
    );
  }

  /**
   * Approve spender to use tokens
   */
  async approveToken(
    tokenAddress: string,
    spenderAddress: string,
    amount: number
  ): Promise<TransactionReceipt | null> {
    if (!this.signer) throw new Error("Wallet not connected");

    const contract = this.getContract(tokenAddress, ERC20TOKEN_ABI, true);
    return this.executeTransaction(
      () => contract.approve(spenderAddress, parseUnits(amount.toString(), 18)),
      "Failed to approve token"
    );
  }

  /**
   * Get token balance
   */
  async getTokenBalance(
    tokenAddress: string,
    userAddress?: string
  ): Promise<string> {
    const address = userAddress || this.address;
    if (!address) throw new Error("No address provided");

    const contract = this.getContract(tokenAddress, ERC20TOKEN_ABI);
    return this.executeViewOperation(async () => {
      const balance = await contract.balanceOf(address);
      return formatUnits(balance, 18);
    }, "Failed to get token balance");
  }
}
