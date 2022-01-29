await new Promise(resolve => setTimeout(resolve, 100));
const fs = require('fs');
const LOCK_CONTRACT_BYTES = fs.readFileSync("../lock_unlock_account_contract/res/lock_unlock_account_latest.wasm");

const _env = (key) => {
  const result = process.env[key];
  if (!result) {
    throw `ENV key '${key}' not set`;
  }
  console.log(`env['${key}']=${result}`);
  return result;
}

const main = async () => {
  const marketplaceAccountId = _env("MARKETPLACE_ACCOUNT");
  const lotAccountId = _env("LOT_ACCOUNT");
  const sellerId = _env("SELLER_ACCOUNT")
  const reservePrice = nearAPI.utils.format.parseNearAmount(_env("RESERVE_PRICE_NEAR"));
  const buyNowPrice = nearAPI.utils.format.parseNearAmount(_env("BUY_NOW_PRICE_NEAR"));
  const duration = (parseInt(_env("DURATION_DAYS"), 10) * 24 * 60 * 60 * 1_000_000_000).toString();

  console.log("CLI LOT OFFER");

  const lotAccount = await near.account(lotAccountId);

  console.log("LOCK...");

  const lockArgs = { "owner_id": marketplaceAccountId };
  const GAS_LOCK = 30_000_000_000_000;
  const NO_DEPOSIT = 0;

  result = await lotAccount.signAndSendTransaction({
    receiverId: lotAccountId,
    actions: [
      nearAPI.transactions.deployContract(LOCK_CONTRACT_BYTES),
      nearAPI.transactions.functionCall("lock", Buffer.from(JSON.stringify(lockArgs)), GAS_LOCK, NO_DEPOSIT),
    ],
  });

  console.log("LOT OFFER...");


  const marketplaceAccount = await near.account(marketplaceAccountId);
  const GAS_LOT_OFFER = 30_000_000_000_000;
  const lotOfferArgs = {
    "seller_id": sellerId,
    "reserve_price": reservePrice,
    "buy_now_price": buyNowPrice,
    "duration": duration,
  };

  result = await lotAccount.signAndSendTransaction({
    receiverId: marketplaceAccountId,
    actions: [
      nearAPI.transactions.functionCall("lot_offer", Buffer.from(JSON.stringify(lotOfferArgs)), GAS_LOCK, NO_DEPOSIT),
    ],
  });

  console.log("DELETING KEYS...");

  const keys = (await lotAccount.getAccessKeys()).map(x => x.public_key);
  console.log(keys);
  result = await lotAccount.signAndSendTransaction({
    receiverId: lotAccountId,
    actions: keys.map(key => nearAPI.transactions.deleteKey(nearAPI.utils.PublicKey.from(key))),
  });

  console.log("FINISHED");
}

await main();
