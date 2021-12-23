import Big from 'big.js'
import * as nearAPI from 'near-api-js';
import React from "react";

export const NETWORK_ID = process.env.REACT_APP_NETWORK_ID || 'default';

export const ACCESS_KEY_ALLOWANCE = Big(1000000000).times(10 ** 24).toFixed();
export const MAX_UINT8 = '340282366920938463463374607431768211455';
export const BOATLOAD_OF_GAS = Big(2).times(10 ** 14).toFixed();

export const toNear = (value = '0') => Big(value).times(10 ** 24).toFixed();
export const nearTo = (value = '0', to = 2) => Big(value).div(10 ** 24).toFixed(to === 0 ? undefined : to);
export const big = (value = '0') => Big(value);
export const tsNear2JS = (time) => Math.floor(time/1000000);

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
  const accountName = accountId && accountId.split('.')[0];
  const accountSuffix = accountId && accountId.split('.')[1];
  return (
    <strong className="account-name">{accountName}<span className="account-suffix">.{accountSuffix}</span></strong>
  )
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