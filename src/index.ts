import { Admin } from "./admin";
import "dotenv/config";

const private_key = process.env.PRIVATE_KEY ? process.env.PRIVATE_KEY : "";
const provider_url = process.env.PROVIDER_URL
  ? process.env.PROVIDER_URL
  : "https://rpc.selendra.org";
const stableCoinAddress = process.env.STABLECOIN_ADDRESS
  ? process.env.STABLECOIN_ADDRESS
  : "0xD21b15d53c11392aD32ABC4Cc56b6002007e208D";
const tokenFactoryAddress = process.env.TOKEN_FACTORY_ADDRESS
  ? process.env.TOKEN_FACTORY_ADDRESS
  : "0xFdD4147c1303bED2875767E793b48Bf8C9db79C9";
const swapAddress = process.env.SWAP_ADDRESS
  ? process.env.SWAP_ADDRESS
  : "0x8ba7982A3978167927D071D6ca6519D4F0FB173B";

async function main() {
  const admin = new Admin(
    provider_url,
    private_key,
    stableCoinAddress,
    tokenFactoryAddress,
    swapAddress
  );

  // // Add to whitelist
  // await admin.addToWhitelist("0x617F2E2fD72FD9D5503197092aC168c91465E7f2");

  // // Remove to whitelist
  // await admin.removeFromWhitelist("0x617F2E2fD72FD9D5503197092aC168c91465E7f2");

  // // Add batch to whitelist
  // await admin.addBatchToWhitelist([
  //   "0x4B20993Bc481177ec7E8f571ceCaE8A9e22C02db",
  //   "0xCA35b7d915458EF540aDe6068dFe2F44E8fa733c",
  // ]);

  // // check whitelist
  // console.log(
  //   await admin.checkWhitelist("0xCA35b7d915458EF540aDe6068dFe2F44E8fa733c")
  // );

  // // set whitelist policy
  // await admin.setWhitelistReceiverPolicy(true);
  // console.log(await admin.checkEnforceWhitelist());

  // // mint stable coin
  // await admin.mintStableCoin("0xCA35b7d915458EF540aDe6068dFe2F44E8fa733c", 100);

  // // check balance
  // console.log(
  //   await admin.checkBalance("0xCA35b7d915458EF540aDe6068dFe2F44E8fa733c")
  // );

  // console.log(await admin.checkTotalSupply());

  // // withdraw stable coin
  // await admin.withdrawStableCoin(
  //   50,
  //   "0xCA35b7d915458EF540aDe6068dFe2F44E8fa733c",
  //   "withdraw"
  // );

  // console.log(
  //   await admin.checkBalance("0xCA35b7d915458EF540aDe6068dFe2F44E8fa733c")
  // );

  // console.log(await admin.checkTotalSupply());

  // // pause/unpase stable coin
  // await admin.pauseContract(stableCoinAddress);
  // await admin.unpauseContract(stableCoinAddress);
  // console.log(await admin.isPausedContract(stableCoinAddress));

  // // create token
  // const addree = await admin.createToken(
  //   "TestPoint",
  //   "TSTA",
  //   "0x8cfc1EeCA441a4554Fc3DFcea1fcBf25749C4ecD",
  //   1
  // );
  // console.log(addree);
  // await admin.addToWhitelist(addree);

  // console.log(await admin.checkWhitelist(addree));

  // console.log(
  //   await admin.isTokenCreatedByFactory(
  //     "0xBd180BD7DBC1FcCd1567EE1E009Ba60dE977EaF8"
  //   )
  // );

  // await admin.transferStableCoin(
  //   "0xBd180BD7DBC1FcCd1567EE1E009Ba60dE977EaF8",
  //   11
  // );

  console.log(
    await admin.mintToken(
      "0xBd180BD7DBC1FcCd1567EE1E009Ba60dE977EaF8",
      "0x3c3134B728b7905F53321Dae63883334b8Dbe2Ac",
      1
    )
  );
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
