import { ethers } from "ethers";

// Interface definitions
export interface Contracts {
  stableCoin: ethers.Contract;
  factory: ethers.Contract;
  tokenSwap: ethers.Contract;
  token: ethers.Contract;
}

export async function setupContracts(
  contracts: Contracts,
  wallet: ethers.Wallet
) {
  const { stableCoin, factory, tokenSwap, token } = contracts;
  const walletAddress = await wallet.getAddress();

  console.log("Setting up contracts...");

  // Add token contract to whitelist
  console.log("Adding addresses to whitelist...");
  const addWhiteList = await stableCoin.batchAddToWhitelist([
    walletAddress,
    await token.getAddress(),
    await tokenSwap.getAddress(),
  ]);

  await addWhiteList.wait();

  // Grant factory minting role to token swap
  console.log("Granting factory minter role to TokenSwap...");
  const FACTORY_MINTER_ROLE = ethers.keccak256(
    ethers.toUtf8Bytes("FACTORY_MINTER_ROLE")
  );
  const grantRole = await factory.grantRole(
    FACTORY_MINTER_ROLE,
    await tokenSwap.getAddress()
  );

  await grantRole.wait();

  // // Approve stablecoin for token swap
  // console.log("Approving StableCoin for TokenSwap...");
  // const approve = await stableCoin.approve(
  //   await tokenSwap.getAddress(),
  //   ethers.parseUnits("1000000", 18) // Approve a large amount
  // );
  // await approve.wait();

  console.log("Setup completed successfully!");
}

export async function createToken(
  factory: ethers.Contract,
  wallet: ethers.Wallet,
  stableCoinAddress: string,
  swapperAddress: string
) {
  // Create a token through the factory
  console.log("Creating a token through the factory...");
  const tx = await factory.createToken(
    "Test Token",
    "TT",
    stableCoinAddress,
    swapperAddress,
    await wallet.getAddress(),
    100
  );
  const receipt = await tx.wait();

  // Extract the token address from the event
  const tokenCreatedEvent = receipt?.logs.find((log: any) => {
    try {
      const parsedLog = factory.interface.parseLog({
        topics: log.topics as string[],
        data: log.data,
      });
      return parsedLog?.name === "TokenCreated";
    } catch {
      return false;
    }
  });

  if (!tokenCreatedEvent) {
    throw new Error("Failed to find TokenCreated event");
  }

  const parsedEvent = factory.interface.parseLog({
    topics: tokenCreatedEvent.topics as string[],
    data: tokenCreatedEvent.data,
  });

  const tokenAddress = parsedEvent?.args[1];
  console.log(`Token created at address: ${tokenAddress}`);

  return tokenAddress;
}

export async function testSwapStableCoinToToken(
  contracts: Contracts,
  wallet: ethers.Wallet
) {
  const { stableCoin, factory, tokenSwap, token } = contracts;
  const walletAddress = await wallet.getAddress();

  console.log("\n--- Testing swapStableCoinToToken ---");

  // Get initial balances
  const initialStableCoinBalance = await stableCoin.balanceOf(walletAddress);
  const initialTokenBalance = await token.balanceOf(walletAddress);

  console.log(
    `Initial StableCoin balance: ${ethers.formatUnits(
      initialStableCoinBalance,
      18
    )}`
  );
  console.log(
    `Initial Token balance: ${ethers.formatUnits(initialTokenBalance, 18)}`
  );

  // Amount to swap
  const stableCoinAmount = ethers.parseUnits("100", 18); // 100 StableCoins
  console.log(
    `Swapping ${ethers.formatUnits(
      stableCoinAmount,
      18
    )} StableCoins for Tokens...`
  );

  // Approve stablecoin for token swap
  console.log("Approving StableCoin for TokenSwap...");
  const approve = await stableCoin.approve(
    await tokenSwap.getAddress(),
    stableCoinAmount
  );
  await approve.wait();

  // Perform swap
  const swapTx = await tokenSwap.swapStableCoinToToken(
    await token.getAddress(),
    stableCoinAmount
  );
  await swapTx.wait();

  // Get final balances
  const finalStableCoinBalance = await stableCoin.balanceOf(walletAddress);
  const finalTokenBalance = await token.balanceOf(walletAddress);
  const tokenContractStableCoinBalance = await stableCoin.balanceOf(
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
    `Token contract StableCoin balance: ${ethers.formatUnits(
      tokenContractStableCoinBalance,
      18
    )}`
  );

  // Verify results
  const stableCoinDifference =
    initialStableCoinBalance - finalStableCoinBalance;
  console.log(
    `StableCoin spent: ${ethers.formatUnits(stableCoinDifference, 18)}`
  );
  console.log(
    `Tokens received: ${ethers.formatUnits(
      finalTokenBalance - initialTokenBalance,
      18
    )}`
  );

  const ratio = await factory.tokenRatios(await token.getAddress());
  const expectedTokens = stableCoinAmount * ratio;
  console.log(`Expected tokens: ${ethers.formatUnits(expectedTokens, 18)}`);

  console.log("swapStableCoinToToken test passed!");
}
