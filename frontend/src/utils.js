import Big from 'big.js'
import * as nearAPI from 'near-api-js';
import React from "react";

export const NETWORK_ID = process.env.REACT_APP_NETWORK_ID || 'default';

export const ACCESS_KEY_ALLOWANCE = Big(1000000000).times(10 ** 24).toFixed();
export const MAX_UINT8 = '340282366920938463463374607431768211455';
export const BOATLOAD_OF_GAS = Big(3).times(10 ** 14).toFixed();

export const toNear = (value = '0') => Big(value).times(10 ** 24).toFixed();
export const nearTo = (value = '0', to = 2) => Big(value).div(10 ** 24).toFixed(to === 0 ? undefined : to);
export const big = (value = '0') => Big(value);
export const tsNear2JS = (time) => Math.floor(time/1000000);

export const LOCK_CONTRACT_HASHES = [
  'CNsF8T5rXcnexk5Ac9Roy6mejRbV7XBQvuXdA2FqnkHj', // v1
  'DKUq738xnns9pKjpv9GifM68UoFSmfnBYNp3hsfkkUFa', // v0
];

export const customRequestSigninFullAccess = async (connection, contractIdOrOptions, successUrl, failureUrl) => {
  let options;
  if (typeof contractIdOrOptions === 'string') {
    console.log('`title` ignored; use `requestSignIn({ contractId, methodNames, successUrl, failureUrl })` instead');
    options = { contractId: contractIdOrOptions, successUrl, failureUrl };
  }
  else {
    options = contractIdOrOptions;
  }
  const currentUrl = new URL(window.location.href);
  const LOGIN_WALLET_URL_SUFFIX = '/login/';
  console.log(connection);
  const newUrl = new URL(connection._walletBaseUrl + LOGIN_WALLET_URL_SUFFIX);
  newUrl.searchParams.set('success_url', options.successUrl || currentUrl.href);
  newUrl.searchParams.set('failure_url', options.failureUrl || currentUrl.href);
  if (options.contractId) {
    /* Throws exception if contract account does not exist */
    // const contractAccount = await connection._near.account(options.contractId);
    // await contractAccount.state();
    // newUrl.searchParams.set('contract_id', options.contractId);
    const accessKey = nearAPI.KeyPair.fromRandom('ed25519');
    newUrl.searchParams.set('public_key', accessKey.getPublicKey().toString());
    const PENDING_ACCESS_KEY_PREFIX = 'pending_key';
    await connection._keyStore.setKey(connection._networkId, PENDING_ACCESS_KEY_PREFIX + accessKey.getPublicKey(), accessKey);
  }
  if (options.methodNames) {
    options.methodNames.forEach(methodName => {
      newUrl.searchParams.append('methodNames', methodName);
    });
  }
  window.location.assign(newUrl.toString());
};

export const renderName = (accountId, suffix = 'testnet') => {
  const accountSuffix = '.' + suffix;
  const accountName = accountId && accountId.split(accountSuffix)[0];
  return (
    <strong className="account-name">{accountName}<span className="account-suffix">{accountSuffix}</span></strong>
  )
}

export function getCountdownTime(lot) {
  return new Date(tsNear2JS(lot.finish_timestamp)).getTime();
}

export function getNextBidAmount(lot) {
  return lot.next_bid_amount ? nearTo(lot.next_bid_amount, 2) : getReservePrice(lot);
}

export function getReservePrice(lot) {
  return nearTo(lot.reserve_price, 2);
}

export function getCurrentPrice(lot) {
  return lot.last_bid_amount ? nearTo(lot.last_bid_amount, 2) : getReservePrice(lot);
}

export function getBuyNowPrice(lot) {
  return lot.buy_now_price ? nearTo(lot.buy_now_price, 2) : getReservePrice(lot);
}

export const fetchBidSafety = async (lot_id, near) => {
  const account = await near.account(lot_id);
  try {
    const codeHash = (await account.state()).code_hash;
    const accessKeysLen = (await account.getAccessKeys()).length;
    const lockerContract = await new nearAPI.Contract(account, lot_id, {
      viewMethods: ['get_owner'],
      changeMethods: []
    });
    const lockerOwner = await lockerContract.get_owner({});
    const balance = (await account.getAccountBalance()).total;
    return { codeHash, accessKeysLen, lockerOwner, balance }
  } catch (e) {
    console.log('check safety error', e)
  }
  return { codeHash: '(unknown)', accessKeysLen: '(unknown)', lockerOwner: '(not found)', balance: 0 }
};

export const loadListPaginated = async (callback, limit = 30) => {
  let result = [];
  let offset = 0;

  while (true) {
    let part = await callback({ limit, offset });
    console.log("paginateContractCall", { limit, offset });
    result.push(...part);
    offset += limit;

    if (part.length < limit) {
      break;
    }
  }

  return result;
};
