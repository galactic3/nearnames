import React from 'react';
import ReactDOM from 'react-dom';
import App from './App';
import getConfig from './config.js';
import * as nearAPI from 'near-api-js';

// Initializing contract
async function initContract() {
  // get network configuration values from config.js
  // based on the network ID we pass to getConfig()
  const nearConfig = getConfig(process.env.NODE_ENV || 'testnet');

  console.log('contractName ' + nearConfig.contractName);

  // create a keyStore for signing transactions using the user's key
  // which is located in the browser local storage after user logs in
  const keyStore = new nearAPI.keyStores.BrowserLocalStorageKeyStore();

  // Initializing connection to the NEAR testnet
  const near = await nearAPI.connect({ keyStore, ...nearConfig });

  // Initialize wallet connection
  const walletConnection = new nearAPI.WalletConnection(near, nearConfig.contractName);

  // Load in user's account data
  let currentUser;
  if (walletConnection.getAccountId()) {
    const balance = await walletConnection.account().getAccountBalance();
    currentUser = {
      accountId: walletConnection.getAccountId(),
      balance: balance.available
    };
  }



  // Initializing our contract APIs by contract name and configuration
  const contract = await new nearAPI.Contract(
    // User's accountId as a string
    walletConnection.account(),
    // accountId of the contract we will be loading
    // NOTE: All contracts on NEAR are deployed to an account and
    // accounts can only have one contract deployed to them.
    nearConfig.contractName,
    {
      // View methods are read-only – they don't modify the state, but usually return some value
      viewMethods: ['lot_list', 'lot_get', 'lot_list_offering_by', 'lot_list_bidding_by', 'profile_get', 'lot_bid_list'],
      // Change methods can modify the state, but you don't receive the returned value when called
      changeMethods: ['lot_offer', 'lot_reoffer', 'lot_bid', 'lot_claim', 'profile_rewards_claim', 'lot_withdraw'],
      // Sender is the account ID to initialize transactions.
      // getAccountId() will return empty string if user is still unauthorized
      sender: walletConnection.getAccountId(),
    }
  );

  return { contract, currentUser, nearConfig, walletConnection, near };
}

window.nearInitPromise = initContract().then(
  ({ contract, currentUser, nearConfig, walletConnection, near }) => {
    ReactDOM.render(
      <App
        contract={contract}
        currentUser={currentUser}
        nearConfig={nearConfig}
        wallet={walletConnection}
        near={near}
      />,
      document.getElementById('root')
    );
  }
);
