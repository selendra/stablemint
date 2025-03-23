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
import { TokenCreatedEvent } from "./interfaces";

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

  async checkBalance(accountAddress: string) {
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

  async checkTotalSupply() {
    try {
      const balance = await this.stableCoin.totalSupply();
      return parseFloat(
        formatUnits ? formatUnits(balance, 18) : balance.toString()
      );
    } catch (error) {
      throw new Error(
        `Failed to check Total Supply: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    }
  }

  async checkTokenBalance(tokenAddress: string, accountAddress: string) {
    try {
      const contract = new Contract(
        tokenAddress,
        ERC20FACTORY_ADMIN_ABI,
        this.provider
      );
      const balance = await contract.balanceOf(accountAddress);
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

  async checkTokenTotalSupply(tokenAddress: string) {
    try {
      const contract = new Contract(
        tokenAddress,
        ERC20FACTORY_ADMIN_ABI,
        this.provider
      );
      const balance = await contract.totalSupply();
      return parseFloat(
        formatUnits ? formatUnits(balance, 18) : balance.toString()
      );
    } catch (error) {
      throw new Error(
        `Failed to check Total Supply: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    }
  }

  async checkEnforceWhitelist() {
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

  async checkWhitelist(accountAddress: string) {
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

  async setWhitelistReceiverPolicy(enforceForReceivers: boolean) {
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

  async addToWhitelist(accountAddress: string) {
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

  async removeFromWhitelist(accountAddress: string) {
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

  async addBatchToWhitelist(accountAddresses: string[]) {
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

  async mintStableCoin(toAddress: string, amount: number) {
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
  ) {
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

  async isTokenCreatedByFactory(tokenAddress: string) {
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
  ) {
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
          } catch (error) {
            console.error("Error parsing log:", error);
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
  ) {
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

  async isPausedContract(contractAddress: string) {
    try {
      const contract = new Contract(
        contractAddress,
        PAUSE_ADMIN_ABI,
        this.provider
      );
      const paused = await contract.paused();
      return paused;
    } catch (error) {
      throw new Error(
        `Failed to check pause status: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    }
  }

  async pauseContract(contractAddress: string) {
    try {
      const contract = new Contract(
        contractAddress,
        PAUSE_ADMIN_ABI,
        this.provider
      );
      const pause = await contract.pause();
      return await pause.wait();
    } catch (error) {
      throw new Error(
        `Failed to pause StableCoin: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    }
  }

  async unpauseContract(contractAddress: string) {
    try {
      const contract = new Contract(
        contractAddress,
        PAUSE_ADMIN_ABI,
        this.provider
      );
      const pause = await contract.unpause();
      return await pause.wait();
    } catch (error) {
      throw new Error(
        `Failed to unpause StableCoin: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    }
  }

  async transferStableCoin(toAddress: string, amount: number) {
    try {
      const transfer = await this.stableCoin.transfer(
        toAddress,
        parseUnits(amount.toString(), 18)
      );
      return await transfer.wait();
    } catch (error) {
      throw new Error(
        `Failed to transfer StableCoin: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    }
  }

  async setFeePercentage(feePercentage: number) {
    try {
      const tx: ContractTransactionResponse =
        await this.tokenSwap.setFeePercentage(feePercentage);
      return await tx.wait();
    } catch (error) {
      throw new Error(
        `Failed to set Fee Percentage: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    }
  }

  async setFeeCollector(feeCollectorAddress: string) {
    try {
      const tx: ContractTransactionResponse =
        await this.tokenSwap.setFeeCollector(feeCollectorAddress);
      return await tx.wait();
    } catch (error) {
      throw new Error(
        `Failed to set Fee Collector: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    }
  }

  async emergencyWithdraw(
    tokenAddress: string,
    amount: number,
    toAddress: string
  ) {
    try {
      const tx: ContractTransactionResponse =
        await this.tokenSwap.emergencyWithdraw(
          tokenAddress,
          parseUnits(amount.toString(), 18),
          toAddress
        );
      return await tx.wait();
    } catch (error) {
      throw new Error(
        `Failed to emergency withdraw: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    }
  }
}
