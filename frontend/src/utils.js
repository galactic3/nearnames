import Big from 'big.js'
import * as nearAPI from 'near-api-js';
import React from "react";
import getConfig from "./config";

export const NETWORK_ID = getConfig(process.env.NODE_ENV || 'testnet').networkId;

export const ACCESS_KEY_ALLOWANCE = Big(1000000000).times(10 ** 24).toFixed();
export const MAX_UINT8 = '340282366920938463463374607431768211455';
export const BOATLOAD_OF_GAS = Big(3).times(10 ** 14).toFixed();

Big.DP = 40;
export const NEAR_ROUND_DIGITS = 2;

export const toNear = (value) => Big(value).times(10 ** 24).round(0, Big.roundDown);
export const nearTo = (value, digits, mode) => {
  // default is 20, need at least 38 for proper rounding of any near balance
  Big.DP = 40;
  return Big(value || '0').div(10 ** 24).toFixed(digits === 0 ? undefined : digits, mode);
};
export const nearToCeil = (value, digits = NEAR_ROUND_DIGITS) => nearTo(value, digits, Big.roundUp);
export const nearToFloor = (value, digits = NEAR_ROUND_DIGITS) => nearTo(value, digits, Big.roundDown);
export const big = (value = '0') => Big(value);
export const tsNear2JS = (time) => Math.floor(time/1000000);

export const LOCK_CONTRACT_HASHES = [
  'CNsF8T5rXcnexk5Ac9Roy6mejRbV7XBQvuXdA2FqnkHj', // v1
  'DKUq738xnns9pKjpv9GifM68UoFSmfnBYNp3hsfkkUFa', // v0
];
export const MIN_RESERVE_PRICE = 0.6;

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

export const renderName = (accountId) => {
  console.log(NETWORK_ID);
  const suffix = NETWORK_ID === 'mainnet' ? 'near' : 'testnet';
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
  return lot.next_bid_amount ? nearToCeil(lot.next_bid_amount) : '';
}

export function getReservePrice(lot) {
  return lot.reserve_price ? nearToCeil(lot.reserve_price) : '';
}

export function getCurrentPrice(lot) {
  return lot.last_bid_amount ? nearToCeil(lot.last_bid_amount) : getReservePrice(lot);
}

export function getBuyNowPrice(lot) {
  return lot.buy_now_price ? nearToCeil(lot.buy_now_price) : '';
}

export const fetchBidSafety = async (lot_id, near, nearConfig) => {
  const account = await near.account(lot_id);
  let isSafe = false;
  try {
    const codeHash = (await account.state()).code_hash;
    const accessKeysLen = (await account.getAccessKeys()).length;
    const lockerContract = await new nearAPI.Contract(account, lot_id, {
      viewMethods: ['get_owner'],
      changeMethods: []
    });
    const lockerOwner = await lockerContract.get_owner({});
    isSafe = LOCK_CONTRACT_HASHES.includes(codeHash) &&
      accessKeysLen === 0 &&
      lockerOwner === nearConfig.contractName;
    console.log(codeHash, accessKeysLen, lockerOwner);
    if (!isSafe) {
      console.log(lot_id + ' account is not safe');
    }
  } catch (e) {
    console.log('check safety error', e)
  }

  return isSafe;
};

export const loadListPaginated = async (callback, limit = 200) => {
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
