import { Contract, ethers } from "ethers";
import { StableCoinABI } from "./abi/stablecoin";
import { ERC20FactoryABI } from "./abi/erc20factory";
import { TokenSwapABI } from "./abi/tokenswap";
import { ERC20TokenABI } from "./abi/erc20token";
import {
  Contracts,
  createToken,
  setupContracts,
  testSwapStableCoinToToken,
} from "./utils";

const StableCoinAddress = "0x0eC1Fcae53BcE5ee89f6487F76985447Dc403518";
const ERC20FactoryAddres = "0x777050fe50078627bD71D2A156FdF4Eba6aAcfF6";
const TokenSwapAddres = "0xB798a9a85b00500DCD60e6ca028D19DCDAAcce70";
const ERC20Token = "0x6264819F433c1364ceB1A84f31a3988591B6Ea8a";
const provider_url = "http://127.0.0.1:9944"; //"https://rpc.selendra.org";

async function getContracts(wallet: ethers.Wallet, tokenAddress?: string) {
  const stableCoin = new Contract(StableCoinAddress, StableCoinABI, wallet);
  const factory = new Contract(ERC20FactoryAddres, ERC20FactoryABI, wallet);
  const tokenSwap = new Contract(TokenSwapAddres, TokenSwapABI, wallet);

  if (!tokenAddress) {
    tokenAddress = await createToken(
      factory,
      wallet,
      StableCoinAddress,
      TokenSwapAddres
    );
  }

  // Connect to the token
  if (!tokenAddress) {
    throw new Error("Token address is undefined");
  }
  const token = new ethers.Contract(tokenAddress, ERC20TokenABI, wallet);

  return { stableCoin, factory, tokenSwap, token };
}

async function testSwapTokenToStableCoin(
  contracts: Contracts,
  wallet: ethers.Wallet
) {
  const { stableCoin, factory, tokenSwap, token } = contracts;
  const walletAddress = await wallet.getAddress();

  console.log("\n--- Testing swapTokenToStableCoin ---");

  // First, we need to ensure the token contract has approved TokenSwap to spend its StableCoins
  // This would typically be done by the token contract's owner
  // For test purposes, we'll transfer some StableCoins to the token contract manually

  // Get initial balances
  const initialStableCoinBalance = await stableCoin.balanceOf(walletAddress);
  const initialTokenBalance = await token.balanceOf(walletAddress);
  const initialTokenContractStableCoinBalance = await stableCoin.balanceOf(
    await token.getAddress()
  );

  console.log(
    `Initial StableCoin balance: ${ethers.formatUnits(
      initialStableCoinBalance,
      18
    )}`
  );
  console.log(
    `Initial Token balance: ${ethers.formatUnits(initialTokenBalance, 18)}`
  );
  console.log(
    `Initial Token contract StableCoin balance: ${ethers.formatUnits(
      initialTokenContractStableCoinBalance,
      18
    )}`
  );

  // Have the token approve the TokenSwap contract to spend its StableCoins
  // We need to use the token contract connection with the owner's wallet
  console.log("Approving TokenSwap to spend Token contract's StableCoins...");
  const approveTx = await stableCoin.approve(
    await tokenSwap.getAddress(),
    ethers.parseUnits("1000000", 18)
  );
  await approveTx.wait();

  // Amount of tokens to swap back
  const tokenAmount = ethers.parseUnits("500", 18); // 1000 Tokens
  console.log(
    `Swapping ${ethers.formatUnits(tokenAmount, 18)} Tokens for StableCoins...`
  );

  // Approve token swap to spend tokens
  const approve = await token.approve(
    await tokenSwap.getAddress(),
    tokenAmount
  );
  await approve.wait();

  // Perform swap
  const swapTx = await tokenSwap.swapTokenToStableCoin(
    await token.getAddress(),
    tokenAmount
  );
  await swapTx.wait();

  // Get final balances
  const finalStableCoinBalance = await stableCoin.balanceOf(walletAddress);
  const finalTokenBalance = await token.balanceOf(walletAddress);
  const finalTokenContractStableCoinBalance = await stableCoin.balanceOf(
    await token.getAddress()
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
    `Final Token contract StableCoin balance: ${ethers.formatUnits(
      finalTokenContractStableCoinBalance,
      18
    )}`
  );

  // Verify results
  const tokenDifference = initialTokenBalance - finalTokenBalance;
  console.log(`Tokens spent: ${ethers.formatUnits(tokenDifference, 18)}`);
  console.log(
    `StableCoins received: ${ethers.formatUnits(
      finalStableCoinBalance - initialStableCoinBalance,
      18
    )}`
  );

  const ratio = await factory.tokenRatios(await token.getAddress());
  const expectedStableCoins = tokenAmount / ratio;
  console.log(
    `Expected StableCoins: ${ethers.formatUnits(expectedStableCoins, 18)}`
  );

  console.log("swapTokenToStableCoin test passed!");
}

async function main(
  isSetupContracts: boolean = true,
  testSwapCoinToken: boolean = true,
  testSwapTokenCoin: boolean = true
) {
  // Connect to provider - use your own RPC URL or local node
  const provider = new ethers.JsonRpcProvider(provider_url);

  // Set up wallet with private key
  const wallet = new ethers.Wallet(
    "0xbdbe387023694b69ff4d565f4f75a818cf347e6175cc1c42bf94b16bfc49f057",
    provider
  );
  const walletAddress = await wallet.getAddress();
  console.log(`Using wallet address: ${walletAddress}`);

  // Get initial balance
  const initialBalance = await provider.getBalance(walletAddress);
  console.log(
    `Initial wallet balance: ${ethers.formatEther(initialBalance)} SEL`
  );

  const contracts: Contracts = await getContracts(
    wallet,
    "0x957B304fdfd7a09662ac4D9C70Ac5B7A02585Dd5"
  );

  if (isSetupContracts) {
    try {
      await setupContracts(contracts, wallet);
    } catch (error) {
      console.error("Setup failed:", error);
    }
  }

  if (testSwapCoinToken) {
    try {
      await testSwapStableCoinToToken(contracts, wallet);
    } catch (error) {
      console.error("Fail to Test swap stablecoin to token:", error);
    }
  }

  if (testSwapTokenCoin) {
    try {
      await testSwapTokenToStableCoin(contracts, wallet);
    } catch (error) {
      console.error("Fail to Test swap stablecoin to token:", error);
    }
  }
}

// Run the main function
main(false, false, true)
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
