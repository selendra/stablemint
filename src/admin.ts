// admin-interactions.ts
// Functions for administrators to interact with the smart contract system
// Updated for ethers v6.13.5

import { ethers } from "ethers";
import {
  Contract,
  Provider,
  Signer,
  ContractTransactionResponse,
  TransactionReceipt,
  formatUnits,
  parseUnits,
  keccak256,
  toUtf8Bytes,
} from "ethers";
import {
  ERC20FACTORY_ADMIN_ABI,
  ERC20TOKEN_ADMIN_ABI,
  ROLE_ADMIN_ABI,
  STABLECOIN_ADMIN_ABI,
  TOKENSWAP_ADMIN_ABI,
} from "./abi";
import { string } from "hardhat/internal/core/params/argumentTypes";
import { TokenCreatedEvent } from "./interfaces";

// Type definitions
type ConnectWalletResult = {
  signer: Signer;
  address: string;
};

// ERC20Token Admin Functions
export async function pauseToken(
  tokenAddress: string,
  signer: Signer
): Promise<TransactionReceipt | null> {
  const contract = new Contract(tokenAddress, ERC20TOKEN_ADMIN_ABI, signer);
  const tx: ContractTransactionResponse = await contract.pause();
  return await tx.wait();
}

export async function unpauseToken(
  tokenAddress: string,
  signer: Signer
): Promise<TransactionReceipt | null> {
  const contract = new Contract(tokenAddress, ERC20TOKEN_ADMIN_ABI, signer);
  const tx: ContractTransactionResponse = await contract.unpause();
  return await tx.wait();
}

// TokenSwap Admin Functions
export async function setFeePercentage(
  swapAddress: string,
  feePercentage: number,
  signer: Signer
): Promise<TransactionReceipt | null> {
  const contract = new Contract(swapAddress, TOKENSWAP_ADMIN_ABI, signer);
  const tx: ContractTransactionResponse = await contract.setFeePercentage(
    feePercentage
  );
  return await tx.wait();
}

export async function setFeeCollector(
  swapAddress: string,
  feeCollectorAddress: string,
  signer: Signer
): Promise<TransactionReceipt | null> {
  const contract = new Contract(swapAddress, TOKENSWAP_ADMIN_ABI, signer);
  const tx: ContractTransactionResponse = await contract.setFeeCollector(
    feeCollectorAddress
  );
  return await tx.wait();
}

export async function pauseSwap(
  swapAddress: string,
  signer: Signer
): Promise<TransactionReceipt | null> {
  const contract = new Contract(swapAddress, TOKENSWAP_ADMIN_ABI, signer);
  const tx: ContractTransactionResponse = await contract.pause();
  return await tx.wait();
}

export async function unpauseSwap(
  swapAddress: string,
  signer: Signer
): Promise<TransactionReceipt | null> {
  const contract = new Contract(swapAddress, TOKENSWAP_ADMIN_ABI, signer);
  const tx: ContractTransactionResponse = await contract.unpause();
  return await tx.wait();
}

export async function emergencyWithdraw(
  swapAddress: string,
  tokenAddress: string,
  amount: number,
  toAddress: string,
  signer: Signer
): Promise<TransactionReceipt | null> {
  const contract = new Contract(swapAddress, TOKENSWAP_ADMIN_ABI, signer);
  const tx: ContractTransactionResponse = await contract.emergencyWithdraw(
    tokenAddress,
    parseUnits(amount.toString(), 18),
    toAddress
  );
  return await tx.wait();
}

export class Admin {
  private provider: ethers.JsonRpcProvider;
  private signer: Signer;
  public stableCoin: ethers.Contract;
  public tokenFactory: ethers.Contract;
  public tokenSwap: ethers.Contract;

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

  // --------------------- check balance StableCoin---------------------------------
  async checkBalance(accountAddress: string): Promise<number> {
    try {
      const balance = await this.stableCoin.balanceOf(accountAddress);
      return parseFloat(
        formatUnits ? formatUnits(balance, 18) : balance.toString()
      );
    } catch (error) {
      throw new Error(
        `Failed to check balance: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    }
  }

  async checkTotalSupply(): Promise<number> {
    try {
      const balance = await this.stableCoin.totalSupply();
      return parseFloat(
        formatUnits ? formatUnits(balance, 18) : balance.toString()
      );
    } catch (error) {
      throw new Error(
        `Failed to check balance: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    }
  }

  // --------------------- Add/remove Whitelist StableCoin-------------------------------------
  async checkEnforceWhitelist(): Promise<boolean> {
    try {
      const enforce = await this.stableCoin.enforceWhitelistForReceivers();
      return enforce;
    } catch (error) {
      throw new Error(
        `Failed to check enforceWhitelistForReceivers: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    }
  }

  async checkWhitelist(accountAddress: string): Promise<boolean> {
    try {
      const isWhiteList = await this.stableCoin.whitelisted(accountAddress);
      return isWhiteList;
    } catch (error) {
      throw new Error(
        `Failed to check WhiteList: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    }
  }

  async setWhitelistReceiverPolicy(
    enforceForReceivers: boolean
  ): Promise<TransactionReceipt | null> {
    try {
      const policy = await this.stableCoin.setWhitelistReceiverPolicy(
        enforceForReceivers
      );
      return await policy.wait();
    } catch (error) {
      throw new Error(
        `Failed to set WhiteList Policy: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    }
  }

  async addToWhitelist(
    accountAddress: string
  ): Promise<TransactionReceipt | null> {
    try {
      const addWhiteList = await this.stableCoin.addToWhitelist(accountAddress);
      return await addWhiteList.wait();
    } catch (error) {
      throw new Error(
        `Failed to add WhiteList: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    }
  }

  async removeFromWhitelist(
    accountAddress: string
  ): Promise<TransactionReceipt | null> {
    try {
      const removeWhiteList = await this.stableCoin.removeFromWhitelist(
        accountAddress
      );
      return await removeWhiteList.wait();
    } catch (error) {
      throw new Error(
        `Failed to remove WhiteList: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    }
  }

  async addBatchToWhitelist(
    accountAddresses: string[]
  ): Promise<TransactionReceipt | null> {
    try {
      const addWhiteList = await this.stableCoin.batchAddToWhitelist(
        accountAddresses
      );
      return await addWhiteList.wait();
    } catch (error) {
      throw new Error(
        `Failed to add batch of WhiteList: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    }
  }

  // -------------------------mint/burn stableCoin---------------------------------

  async mintStableCoin(
    toAddress: string,
    amount: number
  ): Promise<TransactionReceipt | null> {
    try {
      const mint = await this.stableCoin.mint(
        toAddress,
        parseUnits(amount.toString(), 18)
      );
      return await mint.wait();
    } catch (error) {
      throw new Error(
        `Failed to mint StableCoin: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    }
  }

  async withdrawStableCoin(
    amount: number,
    withdrawerAddress: string,
    reason: string
  ): Promise<TransactionReceipt | null> {
    try {
      const withdraw = await this.stableCoin.withdraw(
        parseUnits(amount.toString(), 18),
        withdrawerAddress,
        ethers.encodeBytes32String(reason)
      );
      return await withdraw.wait();
    } catch (error) {
      throw new Error(
        `Failed to withdraw StableCoin: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    }
  }

  // ------------------------- pause/unpause StableCoin---------------------------------

  async isPausedStableCoin(): Promise<boolean> {
    try {
      const paused = await this.stableCoin.paused();
      return paused;
    } catch (error) {
      throw new Error(
        `Failed to check pause status: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    }
  }

  async pauseStableCoin(): Promise<TransactionReceipt | null> {
    try {
      const pause = await this.stableCoin.pause();
      return await pause.wait();
    } catch (error) {
      throw new Error(
        `Failed to pause StableCoin: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    }
  }

  async unpauseStableCoin(): Promise<TransactionReceipt | null> {
    try {
      const pause = await this.stableCoin.unpause();
      return await pause.wait();
    } catch (error) {
      throw new Error(
        `Failed to unpause StableCoin: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    }
  }

  // -------------------------Mint ERC20Factory---------------------------------
  async isTokenCreatedByFactory(tokenAddress: string): Promise<boolean> {
    try {
      const isCreated = await this.tokenFactory.isTokenCreatedByFactory(
        tokenAddress
      );
      return isCreated;
    } catch (error) {
      throw new Error(
        `Failed to check if token created by factory: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    }
  }

  async createToken(
    name: string,
    symbol: string,
    tokenOwner: string,
    tokensPerStableCoin: number
  ): Promise<String> {
    try {
      const create = await this.tokenFactory.createToken(
        name,
        symbol,
        tokenOwner,
        tokensPerStableCoin
      );
      const receipt = await create.wait();

      // Extract token address from event logs
      const logs = receipt.logs || [];
      const tokenCreatedLog = logs.find(
        (log: { topics: string[]; data: any }) => {
          try {
            const parsedLog = this.tokenFactory.interface.parseLog({
              topics: log.topics as string[],
              data: log.data,
            });
            return parsedLog?.name === "TokenCreated";
          } catch {
            return false;
          }
        }
      );

      if (!tokenCreatedLog) {
        throw new Error("TokenCreated event not found in transaction logs");
      }

      const parsedLog = this.tokenFactory.interface.parseLog({
        topics: tokenCreatedLog.topics as string[],
        data: tokenCreatedLog.data,
      }) as unknown as TokenCreatedEvent;

      const newTokenAddress = parsedLog.args.tokenAddress;

      return newTokenAddress;
    } catch (error) {
      throw new Error(
        `Failed to create Token: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    }
  }

  async mintToken(tokenAddress: string, toAddress: string, amount: number) {
    // const stableCoinBalance = await this.stableCoin.balanceOf(tokenAddress);
    // if (stableCoinBalance <= amount) {
    //   throw new Error(
    //     `Failed to mint Token: Stable is to low ${stableCoinBalance}`
    //   );
    // }
    // console.log(stableCoinBalance);

    try {
      const mint = await this.tokenFactory.mintToken(
        tokenAddress,
        toAddress,
        parseUnits(amount.toString(), 18)
      );
      const receipt = await mint.wait();
      if (!receipt) {
        return {
          success: false,
          error: "Transaction receipt undefined",
        };
      }

      const mintEvent = receipt.logs
        .filter((log: { topics: readonly string[]; data: any }) => {
          try {
            const parsed = this.tokenFactory.interface.parseLog({
              topics: Array.isArray(log.topics) ? log.topics : [],
              data: log.data,
            });
            return parsed?.name === "TokenMinted";
          } catch {
            return false;
          }
        })
        .map((log: { topics: readonly string[]; data: any }) => {
          return this.tokenFactory.interface.parseLog({
            topics: Array.isArray(log.topics) ? log.topics : [],
            data: log.data,
          });
        })[0];

      if (!mintEvent) {
        return {
          success: true,
          transactionHash: receipt.hash,
          blockNumber: receipt.blockNumber,
          error: "Transaction successful but TokenMinted event not found",
        };
      }

      return {
        success: true,
        transactionHash: receipt.hash,
      };
    } catch (error) {
      throw new Error(
        `Failed to mint Token: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    }
  }

  // ---------------------------grant/revoke role---------------------

  async grantRole(
    contractAddress: string,
    role: string,
    accountAddress: string
  ) {
    try {
      const contract = new Contract(
        contractAddress,
        ROLE_ADMIN_ABI,
        this.signer
      );
      const roleHash = keccak256(toUtf8Bytes(role));
      const tx: ContractTransactionResponse = await contract.grantRole(
        roleHash,
        accountAddress
      );
      return await tx.wait();
    } catch (error) {
      throw new Error(
        `Failed to Grand Role: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    }
  }

  async revokeRole(
    contractAddress: string,
    role: string,
    accountAddress: string
  ) {
    try {
      const contract = new Contract(
        contractAddress,
        ROLE_ADMIN_ABI,
        this.signer
      );
      const roleHash = keccak256(toUtf8Bytes(role));
      const tx: ContractTransactionResponse = await contract.revokeRole(
        roleHash,
        accountAddress
      );
      return await tx.wait();
    } catch (error) {
      throw new Error(
        `Failed to Rovoke Role: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    }
  }

  async hasRole(
    contractAddress: string,
    role: string,
    accountAddress: string,
    provider: Provider
  ): Promise<boolean> {
    try {
      const contract = new Contract(contractAddress, ROLE_ADMIN_ABI, provider);
      const roleHash = keccak256(toUtf8Bytes(role));
      return await contract.hasRole(roleHash, accountAddress);
    } catch (error) {
      throw new Error(
        `Failed to Get Role: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    }
  }
}
