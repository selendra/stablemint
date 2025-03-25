import { ethers } from "ethers";
import {
  Contract,
  Signer,
  ContractTransactionResponse,
  formatUnits,
  parseUnits,
} from "ethers";
import { StableCoinABI } from "./abi/stablecoin";
import { ERC20FactoryABI } from "./abi/erc20factory";
import { TokenSwapABI } from "./abi/tokenswap";
import { ERC20TokenABI } from "./abi/erc20token";

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
      StableCoinABI,
      this.signer
    );
    this.tokenFactory = new Contract(
      tokenFactoryAddress,
      ERC20FactoryABI,
      this.signer
    );
    this.tokenSwap = new Contract(swapAddress, TokenSwapABI, this.signer);
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
      const token = this.getContract(tokenAddress, ERC20TokenABI);
      const balance = await token.balanceOf(accountAddress);
      return this.formatTokenAmount(balance);
    }, "Failed to check token balance");
  }

  async checkTokenTotalSupply(tokenAddress: string): Promise<number> {
    return this.executeViewOperation(async () => {
      const supply = await this.getContract(
        tokenAddress,
        ERC20TokenABI
      ).totalSupply();
      return this.formatTokenAmount(supply);
    }, "Failed to check token total supply");
  }

  /**
   * StableCoin Whitelist Methods
   */
  async checkWhitelist(accountAddress: string): Promise<boolean> {
    return this.executeViewOperation(
      () => this.stableCoin.whitelisted(accountAddress),
      "Failed to check WhiteList"
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

  async swapperStableToken(tokenAdress: string, amount: number) {
    const swapAddress = await this.tokenSwap.getAddress();
    const signerAddress = await this.signer.getAddress();

    const swapAmonut = parseUnits(amount.toString(), 18);
    const token = this.getContract(tokenAdress, ERC20TokenABI);

    // Get initial balances
    const initialStableCoinBalance = await this.stableCoin.balanceOf(
      signerAddress
    );
    const initialTokenBalance = await token.balanceOf(signerAddress);

    console.log(
      `Initial StableCoin balance: ${ethers.formatUnits(
        initialStableCoinBalance,
        18
      )}`
    );
    console.log(
      `Initial Token balance: ${ethers.formatUnits(initialTokenBalance, 18)}`
    );

    console.log(`Swapping ${swapAmonut} StableCoins for Tokens...`);

    const approve = await this.stableCoin.approve(swapAddress, swapAmonut);
    await approve.wait();

    const swapTx = await this.tokenSwap.swapStableCoinToToken(
      tokenAdress,
      swapAmonut
    );
    await swapTx.wait();

    // Get final balances
    const finalStableCoinBalance = await this.stableCoin.balanceOf(
      signerAddress
    );
    const finalTokenBalance = await token.balanceOf(signerAddress);
    const tokenContractStableCoinBalance = await this.stableCoin.balanceOf(
      tokenAdress
    );

    console.log(
      `Final StableCoin balance: ${ethers.formatUnits(
        finalStableCoinBalance,
        18
      )}`
    );
    console.log(
      `Final Token balance: ${ethers.formatUnits(finalTokenBalance, 18)}`
    );
    console.log(
      `Token contract StableCoin balance: ${ethers.formatUnits(
        tokenContractStableCoinBalance,
        18
      )}`
    );
    console.log(
      `Tokens received: ${ethers.formatUnits(
        finalTokenBalance - initialTokenBalance,
        18
      )}`
    );

    const ratio = await this.tokenFactory.tokenRatios(await token.getAddress());
    const expectedTokens = swapAmonut * ratio;
    console.log(`Expected tokens: ${ethers.formatUnits(expectedTokens, 18)}`);
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
      const stableCoinAddress = await this.stableCoin.getAddress();
      const swapperAddress = await this.tokenSwap.getAddress();
      const tx = await this.tokenFactory.createToken(
        name,
        symbol,
        stableCoinAddress,
        swapperAddress,
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

            const tokenAddress = event.args.tokenAddress;
            const addWhiteList = await this.stableCoin.addToWhitelist(
              tokenAddress
            );

            await addWhiteList.wait();

            return tokenAddress;
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

  async tokenTransfer(tokenAddress: string, to: string, amount: number) {
    const token = this.getContract(tokenAddress, ERC20TokenABI, true);
    const tx = await token.transfer(to, parseUnits(amount.toString(), 18));
    await tx.wait();

    return this.executeTransaction(
      () => token.transfer(to, parseUnits(amount.toString(), 18)),
      "Failed to transfer token"
    );
  }
}
