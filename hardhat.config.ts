import { HardhatUserConfig } from "hardhat/config";
import "@nomicfoundation/hardhat-toolbox";
import "@nomicfoundation/hardhat-ethers";
import "dotenv/config";

// Read environment variables
const PRIVATE_KEY = process.env.PRIVATE_KEY || "";
const USER1 = process.env.USER1_PRIVATE_KEY || "";
const USER2 = process.env.USER2_PRIVATE_KEY || "";
const USER3 = process.env.USER3_PRIVATE_KEY || "";
const INFURA_API_KEY = process.env.INFURA_API_KEY || "";

const config: HardhatUserConfig = {
  solidity: {
    version: "0.8.20",
    settings: {
      optimizer: {
        enabled: true,
        runs: 200,
        details: {
          yul: true, 
          yulDetails: {
            stackAllocation: true,
            optimizerSteps: "dhfoDgvulfnTUtnIf" 
          },
        },
      },
      viaIR: true,
    },
  },
  networks: {
    hardhat: {},
    localhost: {
      url: "http://127.0.0.1:9944",
      accounts: [PRIVATE_KEY, USER1, USER2, USER3],
    },
    selendra: {
      url: "https://rpc.selendra.org",
      accounts: [PRIVATE_KEY, USER1, USER2, USER3],
    },
    sepolia: {
      url: `https://sepolia.infura.io/v3/${INFURA_API_KEY}`,
      accounts: [PRIVATE_KEY],
    },
  },
  paths: {
    sources: "./contracts",
    tests: "./test",
    cache: "./cache",
    artifacts: "./artifacts",
  },
  typechain: {
    outDir: "typechain",
    target: "ethers-v6",
  }
};

export default config;