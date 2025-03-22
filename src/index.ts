import { Admin } from "./admin";
import "dotenv/config";

const private_key = process.env.PRIVATE_KEY ? process.env.PRIVATE_KEY : "";
const provider_url = process.env.PROVIDER_URL
  ? process.env.PROVIDER_URL
  : "https://rpc.selendra.org";
const stableCoinAddress = process.env.STABLECOIN_ADDRESS
  ? process.env.STABLECOIN_ADDRESS
  : "0xb7B9838d9d37444e50B08fCdf5Db2887d5D560d4";
const tokenFactoryAddress = process.env.TOKEN_FACTORY_ADDRESS
  ? process.env.TOKEN_FACTORY_ADDRESS
  : "0xd7128352dA44c6f6f8D92B7945C446F493e22849";
const swapAddress = process.env.SWAP_ADDRESS
  ? process.env.SWAP_ADDRESS
  : "0x02f0FEBea10b76574D05F8feF36b122FE1A0089e";

async function main() {
  const admin = new Admin(
    provider_url,
    private_key,
    stableCoinAddress,
    tokenFactoryAddress,
    swapAddress
  );

  // // Add to whitelist
  // const receipt = await admin.addToWhitelist(
  //   "0x617F2E2fD72FD9D5503197092aC168c91465E7f2"
  // );
  // console.log(receipt);

  // // Remove to whitelist
  // await admin.removeFromWhitelist("0x617F2E2fD72FD9D5503197092aC168c91465E7f2");

  // // Add batch to whitelist
  // await admin.addBatchToWhitelist([
  //   "0x4B20993Bc481177ec7E8f571ceCaE8A9e22C02db",
  //   "0xCA35b7d915458EF540aDe6068dFe2F44E8fa733c",
  // ]);

  // // set whitelist policy
  // await admin.setWhitelistReceiverPolicy(true);
  // console.log(await admin.checkEnforceWhitelist());

  // // check whitelist
  // console.log(
  //   await admin.checkWhitelist("0xCA35b7d915458EF540aDe6068dFe2F44E8fa733c")
  // );

  // // mint stable coin
  // await admin.mintStableCoin("0xCA35b7d915458EF540aDe6068dFe2F44E8fa733c", 100);

  // // check balance
  // console.log(
  //   await admin.checkBalance("0xCA35b7d915458EF540aDe6068dFe2F44E8fa733c")
  // );

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
  // await admin.pauseStableCoin();
  // await admin.unpauseStableCoin();
  // console.log(await admin.isPausedStableCoin());

  // await admin.mintToken();
  // const res = await admin.createToken(
  //   "Test",
  //   "TST",
  //   "0x617F2E2fD72FD9D5503197092aC168c91465E7f2",
  //   1000
  // );
  // console.log(res);

  // console.log(
  //   await admin.isTokenCreatedByFactory(
  //     "0x88335B165A47020C14726f18bFD5DfD567586b26"
  //   )
  // );

  console.log(
    await admin.mintToken(
      "0x88335B165A47020C14726f18bFD5DfD567586b26",
      "0x3c3134B728b7905F53321Dae63883334b8Dbe2Ac",
      1000000
    )
  );
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
