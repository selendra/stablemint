import { ethers } from "hardhat";

async function main() {
  const [deployer] = await ethers.getSigners();

  console.log("Deploying contracts with account:", deployer.address);
  console.log("Account balance:", (await ethers.provider.getBalance(deployer.address)).toString());
  console.log("Current time:", new Date().toISOString());

  // Deploy Whitelist with whitelisting enabled
  console.log("\n1. Deploying Whitelist...");
  const Whitelist = await ethers.getContractFactory("Whitelist");
  const whitelist = await Whitelist.deploy(true); // Enable whitelisting on deployment
  await whitelist.waitForDeployment();
  const whitelistAddress = await whitelist.getAddress();
  console.log("   Whitelist deployed to:", whitelistAddress);

  // Deploy TransferLimiter
  console.log("\n2. Deploying TransferLimiter...");
  const TransferLimiter = await ethers.getContractFactory("TransferLimiter");
  const transferLimiter = await TransferLimiter.deploy();
  await transferLimiter.waitForDeployment();
  const transferLimiterAddress = await transferLimiter.getAddress();
  console.log("   TransferLimiter deployed to:", transferLimiterAddress);

  // Deploy StableCoin with links to utility contracts
  console.log("\n3. Deploying StableCoin...");
  const name = "Optimized Stable Coin";
  const symbol = "OSC";
  const initialSupply = 1000000n; // 1 million tokens initially
  
  const StableCoin = await ethers.getContractFactory("StableCoin");
  const stableCoin = await StableCoin.deploy(
    name, 
    symbol, 
    initialSupply,
    whitelistAddress,
    transferLimiterAddress
  );
  await stableCoin.waitForDeployment();
  const stableCoinAddress = await stableCoin.getAddress();
  console.log("   StableCoin deployed to:", stableCoinAddress);

  // Configure TransferLimiter
  console.log("\n4. Configuring TransferLimiter with default limits...");
  
  // Create the LimitConfig struct
  const limitConfig = {
    maxTransferAmount: ethers.parseUnits("10000", 18),  // 10,000 tokens max per transfer
    cooldownPeriod: 60n,                                // 1 minute cooldown between transfers
    periodLimit: ethers.parseUnits("50000", 18),        // 50,000 tokens per period
    periodDuration: 86400n                              // 24 hour period duration
  };
  
  await transferLimiter.setAllDefaultLimits(stableCoinAddress, limitConfig);
  console.log("   Default transfer limits configured");
  
  // Whitelist the deployer and exempt from limits
  console.log("\n5. Setting up whitelisting and exemptions...");
  await transferLimiter.setExemption(stableCoinAddress, deployer.address, true);
  console.log("   Deployer exempted from transfer limits");
  
  // Add the StableCoin contract to whitelist for authorization
  console.log("\n6. Finalizing permissions...");
  await whitelist.authorizeContract(stableCoinAddress);
  console.log("   StableCoin authorized in Whitelist");
  await transferLimiter.authorizeContract(stableCoinAddress);
  console.log("   StableCoin authorized in TransferLimiter");

  console.log("\nDeployment complete! Summary:");
  console.log(`- Whitelist:       ${whitelistAddress}`);
  console.log(`- TransferLimiter: ${transferLimiterAddress}`);
  console.log(`- StableCoin:      ${stableCoinAddress} (${symbol})`);
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });