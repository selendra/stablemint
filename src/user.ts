// user-interactions.ts
// Functions for regular users to interact with the smart contract system
// Updated for ethers v6.13.5

import { ethers } from 'ethers';
import {
  Contract,
  Provider,
  JsonRpcProvider,
  BrowserProvider,
  Signer,
  ContractTransactionReceipt,
  ContractTransactionResponse,
  TransactionReceipt,
  formatUnits,
  parseUnits
} from 'ethers';
import { ERC20FACTORY_ABI, ERC20TOKEN_ABI, STABLECOIN_ABI, TOKENSWAP_ABI } from './abi';

// Type definitions
type ConnectWalletResult = {
  signer: Signer;
  address: string;
};

// Connect to provider and signer
export async function connectWallet(provider: any): Promise<ConnectWalletResult> {
    const browserProvider = new BrowserProvider(provider);
    await browserProvider.send('eth_requestAccounts', []);
    const signer = await browserProvider.getSigner();
    const address = await signer.getAddress();
    return { signer, address };
  }

  // StableCoin User Functions
export async function checkWhitelistStatus(stableCoinAddress: string, userAddress: string, provider: Provider): Promise<boolean> {
    const contract = new Contract(stableCoinAddress, STABLECOIN_ABI, provider);
    return await contract.whitelisted(userAddress);
  }
  
  export async function transferStableCoin(stableCoinAddress: string, toAddress: string, amount: number, signer: Signer): Promise<TransactionReceipt | null> {
    const contract = new Contract(stableCoinAddress, STABLECOIN_ABI, signer);
    const tx: ContractTransactionResponse = await contract.transfer(toAddress, parseUnits(amount.toString(), 18));
    return await tx.wait();
  }
  
  export async function approveStableCoin(stableCoinAddress: string, spenderAddress: string, amount: number, signer: Signer): Promise<TransactionReceipt | null> {
    const contract = new Contract(stableCoinAddress, STABLECOIN_ABI, signer);
    const tx: ContractTransactionResponse = await contract.approve(spenderAddress, parseUnits(amount.toString(), 18));
    return await tx.wait();
  }
  
  export async function getStableCoinBalance(stableCoinAddress: string, userAddress: string, provider: Provider): Promise<string> {
    const contract = new Contract(stableCoinAddress, STABLECOIN_ABI, provider);
    const balance = await contract.balanceOf(userAddress);
    return formatUnits(balance, 18);
  }
  
  // ERC20Token User Functions
  export async function transferToken(tokenAddress: string, toAddress: string, amount: number, signer: Signer): Promise<TransactionReceipt | null> {
    const contract = new Contract(tokenAddress, ERC20TOKEN_ABI, signer);
    const tx: ContractTransactionResponse = await contract.transfer(toAddress, parseUnits(amount.toString(), 18));
    return await tx.wait();
  }
  
  export async function approveToken(tokenAddress: string, spenderAddress: string, amount: number, signer: Signer): Promise<TransactionReceipt | null> {
    const contract = new Contract(tokenAddress, ERC20TOKEN_ABI, signer);
    const tx: ContractTransactionResponse = await contract.approve(spenderAddress, parseUnits(amount.toString(), 18));
    return await tx.wait();
  }
  
  export async function getTokenBalance(tokenAddress: string, userAddress: string, provider: Provider): Promise<string> {
    const contract = new Contract(tokenAddress, ERC20TOKEN_ABI, provider);
    const balance = await contract.balanceOf(userAddress);
    return formatUnits(balance, 18);
  }
  
  // TokenSwap User Functions
  export async function swapStableCoinToToken(swapAddress: string, tokenAddress: string, stableCoinAmount: number, signer: Signer): Promise<TransactionReceipt | null> {
    const contract = new Contract(swapAddress, TOKENSWAP_ABI, signer);
    const tx: ContractTransactionResponse = await contract.swapStableCoinToToken(
      tokenAddress, 
      parseUnits(stableCoinAmount.toString(), 18)
    );
    return await tx.wait();
  }
  
  export async function swapTokenToStableCoin(swapAddress: string, tokenAddress: string, tokenAmount: number, signer: Signer): Promise<TransactionReceipt | null> {
    const contract = new Contract(swapAddress, TOKENSWAP_ABI, signer);
    const tx: ContractTransactionResponse = await contract.swapTokenToStableCoin(
      tokenAddress, 
      parseUnits(tokenAmount.toString(), 18)
    );
    return await tx.wait();
  }
  
  // System Information Functions
  export async function getTokenRatio(factoryAddress: string, tokenAddress: string, provider: Provider): Promise<string> {
    const contract = new Contract(factoryAddress, ERC20FACTORY_ABI, provider);
    const ratio = await contract.tokenRatios(tokenAddress);
    return ratio.toString();
  }
  
  export async function getFeePercentage(swapAddress: string, provider: Provider): Promise<string> {
    const contract = new Contract(swapAddress, TOKENSWAP_ABI, provider);
    const fee = await contract.feePercentage();
    return fee.toString();
  }
  
  export async function isSystemPaused(stableCoinAddress: string, provider: Provider): Promise<boolean> {
    const contract = new Contract(stableCoinAddress, STABLECOIN_ABI, provider);
    return await contract.paused();
  }