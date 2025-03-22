// admin-interactions.ts
// Functions for administrators to interact with the smart contract system
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
  parseUnits,
  keccak256,
  toUtf8Bytes,
} from 'ethers';
import { ERC20FACTORY_ADMIN_ABI, ERC20TOKEN_ADMIN_ABI, ROLE_ADMIN_ABI, STABLECOIN_ADMIN_ABI, TOKENSWAP_ADMIN_ABI } from './abi';

// Type definitions
type ConnectWalletResult = {
  signer: Signer;
  address: string;
};

// Connect to provider and signer
export async function connectAdminWallet(provider: any): Promise<ConnectWalletResult> {
  const browserProvider = new BrowserProvider(provider);
  await browserProvider.send('eth_requestAccounts', []);
  const signer = await browserProvider.getSigner();
  const address = await signer.getAddress();
  return { signer, address };
}

// StableCoin Admin Functions
export async function addToWhitelist(stableCoinAddress: string, accountAddress: string, signer: Signer): Promise<TransactionReceipt | null> {
  const contract = new Contract(stableCoinAddress, STABLECOIN_ADMIN_ABI, signer);
  const tx: ContractTransactionResponse = await contract.addToWhitelist(accountAddress);
  return await tx.wait();
}

export async function removeFromWhitelist(stableCoinAddress: string, accountAddress: string, signer: Signer): Promise<TransactionReceipt | null> {
  const contract = new Contract(stableCoinAddress, STABLECOIN_ADMIN_ABI, signer);
  const tx: ContractTransactionResponse = await contract.removeFromWhitelist(accountAddress);
  return await tx.wait();
}

export async function batchAddToWhitelist(stableCoinAddress: string, accountAddresses: string[], signer: Signer): Promise<TransactionReceipt | null> {
  const contract = new Contract(stableCoinAddress, STABLECOIN_ADMIN_ABI, signer);
  const tx: ContractTransactionResponse = await contract.batchAddToWhitelist(accountAddresses);
  return await tx.wait();
}

export async function setWhitelistReceiverPolicy(stableCoinAddress: string, enforceForReceivers: boolean, signer: Signer): Promise<TransactionReceipt | null> {
  const contract = new Contract(stableCoinAddress, STABLECOIN_ADMIN_ABI, signer);
  const tx: ContractTransactionResponse = await contract.setWhitelistReceiverPolicy(enforceForReceivers);
  return await tx.wait();
}

export async function mintStableCoin(stableCoinAddress: string, toAddress: string, amount: number, signer: Signer): Promise<TransactionReceipt | null> {
  const contract = new Contract(stableCoinAddress, STABLECOIN_ADMIN_ABI, signer);
  const tx: ContractTransactionResponse = await contract.mint(toAddress, parseUnits(amount.toString(), 18));
  return await tx.wait();
}

export async function burnStableCoin(stableCoinAddress: string, amount: number, signer: Signer): Promise<TransactionReceipt | null> {
  const contract = new Contract(stableCoinAddress, STABLECOIN_ADMIN_ABI, signer);
  const tx: ContractTransactionResponse = await contract.burn(parseUnits(amount.toString(), 18));
  return await tx.wait();
}

export async function withdrawStableCoin(stableCoinAddress: string, amount: number, withdrawerAddress: string, data: string, signer: Signer): Promise<TransactionReceipt | null> {
  const contract = new Contract(stableCoinAddress, STABLECOIN_ADMIN_ABI, signer);
  const tx: ContractTransactionResponse = await contract.withdraw(
    parseUnits(amount.toString(), 18),
    withdrawerAddress,
    ethers.encodeBytes32String(data)
  );
  return await tx.wait();
}

export async function pauseStableCoin(stableCoinAddress: string, signer: Signer): Promise<TransactionReceipt | null> {
  const contract = new Contract(stableCoinAddress, STABLECOIN_ADMIN_ABI, signer);
  const tx: ContractTransactionResponse = await contract.pause();
  return await tx.wait();
}

export async function unpauseStableCoin(stableCoinAddress: string, signer: Signer): Promise<TransactionReceipt | null> {
  const contract = new Contract(stableCoinAddress, STABLECOIN_ADMIN_ABI, signer);
  const tx: ContractTransactionResponse = await contract.unpause();
  return await tx.wait();
}

// ERC20Token Admin Functions
export async function pauseToken(tokenAddress: string, signer: Signer): Promise<TransactionReceipt | null> {
  const contract = new Contract(tokenAddress, ERC20TOKEN_ADMIN_ABI, signer);
  const tx: ContractTransactionResponse = await contract.pause();
  return await tx.wait();
}

export async function unpauseToken(tokenAddress: string, signer: Signer): Promise<TransactionReceipt | null> {
  const contract = new Contract(tokenAddress, ERC20TOKEN_ADMIN_ABI, signer);
  const tx: ContractTransactionResponse = await contract.unpause();
  return await tx.wait();
}

// ERC20Factory Admin Functions
export async function createToken(
  factoryAddress: string, 
  name: string, 
  symbol: string, 
  tokenOwner: string, 
  tokensPerStableCoin: number, 
  signer: Signer
): Promise<TransactionReceipt | null> {
  const contract = new Contract(factoryAddress, ERC20FACTORY_ADMIN_ABI, signer);
  const tx: ContractTransactionResponse = await contract.createToken(name, symbol, tokenOwner, tokensPerStableCoin);
  return await tx.wait();
}

export async function mintToken(
  factoryAddress: string, 
  tokenAddress: string, 
  toAddress: string, 
  amount: number, 
  signer: Signer
): Promise<TransactionReceipt | null> {
  const contract = new Contract(factoryAddress, ERC20FACTORY_ADMIN_ABI, signer);
  const tx: ContractTransactionResponse = await contract.mintToken(tokenAddress, toAddress, parseUnits(amount.toString(), 18));
  return await tx.wait();
}

export async function setStableCoinAddress(
  factoryAddress: string, 
  stableCoinAddress: string, 
  signer: Signer
): Promise<TransactionReceipt | null> {
  const contract = new Contract(factoryAddress, ERC20FACTORY_ADMIN_ABI, signer);
  const tx: ContractTransactionResponse = await contract.setStableCoinAddress(stableCoinAddress);
  return await tx.wait();
}

// TokenSwap Admin Functions
export async function setFeePercentage(
  swapAddress: string, 
  feePercentage: number, 
  signer: Signer
): Promise<TransactionReceipt | null> {
  const contract = new Contract(swapAddress, TOKENSWAP_ADMIN_ABI, signer);
  const tx: ContractTransactionResponse = await contract.setFeePercentage(feePercentage);
  return await tx.wait();
}

export async function setFeeCollector(
  swapAddress: string, 
  feeCollectorAddress: string, 
  signer: Signer
): Promise<TransactionReceipt | null> {
  const contract = new Contract(swapAddress, TOKENSWAP_ADMIN_ABI, signer);
  const tx: ContractTransactionResponse = await contract.setFeeCollector(feeCollectorAddress);
  return await tx.wait();
}

export async function pauseSwap(swapAddress: string, signer: Signer): Promise<TransactionReceipt | null> {
  const contract = new Contract(swapAddress, TOKENSWAP_ADMIN_ABI, signer);
  const tx: ContractTransactionResponse = await contract.pause();
  return await tx.wait();
}

export async function unpauseSwap(swapAddress: string, signer: Signer): Promise<TransactionReceipt | null> {
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

// Role Management Functions
export async function grantRole(
  contractAddress: string, 
  role: string, 
  accountAddress: string, 
  signer: Signer
): Promise<TransactionReceipt | null> {
  const contract = new Contract(contractAddress, ROLE_ADMIN_ABI, signer);
  const roleHash = keccak256(toUtf8Bytes(role));
  const tx: ContractTransactionResponse = await contract.grantRole(roleHash, accountAddress);
  return await tx.wait();
}

export async function revokeRole(
  contractAddress: string, 
  role: string, 
  accountAddress: string, 
  signer: Signer
): Promise<TransactionReceipt | null> {
  const contract = new Contract(contractAddress, ROLE_ADMIN_ABI, signer);
  const roleHash = keccak256(toUtf8Bytes(role));
  const tx: ContractTransactionResponse = await contract.revokeRole(roleHash, accountAddress);
  return await tx.wait();
}

export async function hasRole(
  contractAddress: string, 
  role: string, 
  accountAddress: string, 
  provider: Provider
): Promise<boolean> {
  const contract = new Contract(contractAddress, ROLE_ADMIN_ABI, provider);
  const roleHash = keccak256(toUtf8Bytes(role));
  return await contract.hasRole(roleHash, accountAddress);
}
