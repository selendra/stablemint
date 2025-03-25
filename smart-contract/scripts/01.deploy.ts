const { ethers } = require("hardhat");

async function main() {
  console.log("Deploying TokenSwap contract...");

  // Get deployer account
  const [deployer] = await ethers.getSigners();
  console.log("Deploying contracts with the account:", deployer.address);
  console.log(
    "Account balance:",
    (await ethers.provider.getBalance(deployer.address)).toString()
  );

  // Get contract factories
  const StableCoin = await ethers.getContractFactory("StableCoin");
  const ERC20Factory = await ethers.getContractFactory("ERC20Factory");
  const TokenSwap = await ethers.getContractFactory("TokenSwap");

  // Deploy StableCoin contract
  console.log("Deploying StableCoin...");
  const stableCoin = await StableCoin.deploy("KHMER_RIEL", "KHR", 1000000);
  await stableCoin.waitForDeployment();
  const stableCoinAddress = await stableCoin.getAddress();
  console.log("StableCoin deployed to:", stableCoinAddress);

  // Assign StableCoin roles
  const MINTER_ROLE = ethers.keccak256(ethers.toUtf8Bytes("MINTER_ROLE"));
  const WHITELIST_MANAGER_ROLE = ethers.keccak256(
    ethers.toUtf8Bytes("WHITELIST_MANAGER_ROLE")
  );

  console.log("Setting up StableCoin roles...");
  await stableCoin.grantRole(MINTER_ROLE, deployer.address);
  await stableCoin.grantRole(WHITELIST_MANAGER_ROLE, deployer.address);

  // Whitelist the deployer
  await stableCoin.addToWhitelist(deployer.address);

  // Deploy ERC20Factory contract
  console.log("Deploying ERC20Factory...");
  const factory = await ERC20Factory.deploy(
    deployer.address, // Owner
    stableCoinAddress // StableCoin address
  );
  await factory.waitForDeployment();
  const factoryAddress = await factory.getAddress();
  console.log("ERC20Factory deployed to:", factoryAddress);

  // Set up Factory roles
  const TOKEN_CREATOR_ROLE = ethers.keccak256(
    ethers.toUtf8Bytes("TOKEN_CREATOR_ROLE")
  );
  const FACTORY_MINTER_ROLE = ethers.keccak256(
    ethers.toUtf8Bytes("FACTORY_MINTER_ROLE")
  );

  console.log("Setting up Factory roles...");
  await factory.grantRole(TOKEN_CREATOR_ROLE, deployer.address);
  await factory.grantRole(FACTORY_MINTER_ROLE, deployer.address);

  // Set up parameters for TokenSwap
  const adminAddress = deployer.address; // Using deployer as admin

  // Deploy TokenSwap
  console.log("Deploying TokenSwap...");
  const tokenSwap = await TokenSwap.deploy(
    stableCoinAddress,
    factoryAddress,
    adminAddress
  );
  await tokenSwap.waitForDeployment();
  const tokenSwapAddress = await tokenSwap.getAddress();
  console.log("TokenSwap deployed to:", tokenSwapAddress);

  // Whitelist the TokenSwap contract
  console.log("Whitelisting TokenSwap contract...");
  await stableCoin.addToWhitelist(tokenSwapAddress);

  // Fund the TokenSwap contract with StableCoin
  console.log("Funding TokenSwap with StableCoin...");
  const fundAmount = ethers.parseUnits("100000", 18); // 100,000 tokens with 18 decimals
  await stableCoin.mint(tokenSwapAddress, fundAmount);

  // Grant factory minting role to token swap
  console.log("Granting factory minter role to TokenSwap...");
  await factory.grantRole(FACTORY_MINTER_ROLE, await tokenSwap.getAddress());

  console.log("\nDeployment Summary:");
  console.log("------------------");
  console.log("StableCoin:", stableCoinAddress);
  console.log("ERC20Factory:", factoryAddress);
  console.log("TokenSwap:", tokenSwapAddress);
}

// Run the deployment
main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
