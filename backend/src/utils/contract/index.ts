export { Admin } from "./admin";
export { User } from "./user";

import { Admin } from "./admin";
import "dotenv/config";
import { User } from "./user";
import { Contract, ethers, parseUnits } from "ethers";
import { TOKENSWAP_ABI } from "./abi";

const private_key = process.env.PRIVATE_KEY ? process.env.PRIVATE_KEY : "";
const provider_url = process.env.PROVIDER_URL
  ? process.env.PROVIDER_URL
  : "https://rpc.selendra.org";
const stableCoinAddress = process.env.STABLECOIN_ADDRESS
  ? process.env.STABLECOIN_ADDRESS
  : "";
const tokenFactoryAddress = process.env.TOKEN_FACTORY_ADDRESS
  ? process.env.TOKEN_FACTORY_ADDRESS
  : "";
const swapAddress = process.env.SWAP_ADDRESS ? process.env.SWAP_ADDRESS : "";

async function main() {
  // const admin = new Admin(
  //   provider_url,
  //   private_key,
  //   stableCoinAddress,
  //   tokenFactoryAddress,
  //   swapAddress
  // );

  //   // // Add to whitelist
  //   // await admin.addToWhitelist("0x617F2E2fD72FD9D5503197092aC168c91465E7f2");

  //   // // Remove to whitelist
  //   // await admin.removeFromWhitelist("0x617F2E2fD72FD9D5503197092aC168c91465E7f2");

  //   // // Add batch to whitelist
  //   // await admin.addBatchToWhitelist([
  //   //   "0x4B20993Bc481177ec7E8f571ceCaE8A9e22C02db",
  //   //   "0xCA35b7d915458EF540aDe6068dFe2F44E8fa733c",
  //   // ]);

  //   // // check whitelist
  //   // console.log(
  //   //   await admin.checkWhitelist("0xCA35b7d915458EF540aDe6068dFe2F44E8fa733c")
  //   // );

  //   // // set whitelist policy
  //   // await admin.setWhitelistReceiverPolicy(true);
  //   // console.log(await admin.checkEnforceWhitelist());

  //   // // mint stable coin
  //   // await admin.mintStableCoin("0xCA35b7d915458EF540aDe6068dFe2F44E8fa733c", 100);

  //   // check balance
  //   // console.log(
  //   //   await admin.checkBalance("0xBd180BD7DBC1FcCd1567EE1E009Ba60dE977EaF8")
  //   // );

  //   // console.log(await admin.checkTotalSupply());

  //   // // withdraw stable coin
  //   // await admin.withdrawStableCoin(
  //   //   50,
  //   //   "0xCA35b7d915458EF540aDe6068dFe2F44E8fa733c",
  //   //   "withdraw"
  //   // );

  //   // console.log(
  //   //   await admin.checkBalance("0xCA35b7d915458EF540aDe6068dFe2F44E8fa733c")
  //   // );

  //   // console.log(await admin.checkTotalSupply());

  //   // // pause/unpase stable coin
  //   // await admin.pauseContract(stableCoinAddress);
  //   // await admin.unpauseContract(stableCoinAddress);
  //   // console.log(await admin.isPausedContract(stableCoinAddress));

  //   // // create token
  //   // const addree = await admin.createToken(
  //   //   "TestPoint",
  //   //   "TSTA",
  //   //   "0x8cfc1EeCA441a4554Fc3DFcea1fcBf25749C4ecD",
  //   //   1
  //   // );
  //   // console.log(addree);
  //   // await admin.addToWhitelist(await admin.signer.getAddress());
  //   // await admin.addToWhitelist(addree);

  //   // console.log(await admin.checkWhitelist(addree));

  //   // console.log(
  //   //   await admin.isTokenCreatedByFactory(
  //   //     "0xBd180BD7DBC1FcCd1567EE1E009Ba60dE977EaF8"
  //   //   )
  //   // );

  //   await admin.transferStableCoin(
  //     "0x2402Ed00D1223500bA3B45fa30549Be28Dbe50B3",
  //     11
  //   );

  //   console.log(
  //     await admin.mintToken(
  //       "0x2402Ed00D1223500bA3B45fa30549Be28Dbe50B3",
  //       "0x3c3134B728b7905F53321Dae63883334b8Dbe2Ac",
  //       100
  //     )
  //   );

  //   // console.log(await admin.getAllCreatedTokens());

  const provider = new ethers.JsonRpcProvider("https://rpc.selendra.org");
  const signer = new ethers.Wallet(
    "0x42b960b83e2f57635f90b42ddb910c6976dbff409236baf89d922b04c183bef4",
    provider
  );

  const swapContract = new Contract(
    "0x1E7060DDEDEeA59E255EFD169d30971464eEC590",
    TOKENSWAP_ABI,
    signer
  );

  const amount = 20;
  const res = await swapContract.swapTokenToStableCoin(
    "0x2402Ed00D1223500bA3B45fa30549Be28Dbe50B3",
    parseUnits(amount.toString(), 18)
  );
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
