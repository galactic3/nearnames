import 'regenerator-runtime/runtime';
import React, { useState, useEffect } from 'react';
import PropTypes from 'prop-types';
import Big from 'big.js';
import Form from './components/Form';
import SignIn from './components/SignIn';
import Lots from './components/Lots';

const App = ({ contract, currentUser, nearConfig, wallet }) => {
  const [lots, setLots] = useState([]);

  console.log(contract);

  useEffect(() => {
    contract.lot_list().then(setLots);
  }, []);

  console.log(lots);

  const onSubmit = (e) => {
    e.preventDefault();

    const { fieldset, seller_id, reserve_price, buy_now_price, duration } = e.target.elements;

    console.log(seller_id.value,
      Big(reserve_price.value || '0').times(10 ** 24).toFixed(),
      Big(buy_now_price.value || '0').times(10 ** 24).toFixed(),
      Big(duration.value).times(3600000000000).toFixed());

    fieldset.disabled = true;

    // update blockchain data in background
    contract.lot_offer(
      {
        seller_id: seller_id.value,
        reserve_price: Big(reserve_price.value || '0').times(10 ** 24).toFixed(),
        buy_now_price: Big(buy_now_price.value || '0').times(10 ** 24).toFixed(),
        duration: duration.value*3600000000000
      }
      // BOATLOAD_OF_GAS,
      // Big(reserve_price.value || '0').times(10 ** 24).toFixed()
    ).then(() => {
      contract.lot_list().then(lots => {
        setLots(lots);
        seller_id.value = '';
        reserve_price.value = '1,5';
        fieldset.disabled = false;
        message.focus();
      });
    });
  };

  const signIn = () => {
    wallet.requestSignIn(
      nearConfig.contractName,
      'Name hub'
    );
  };

  const signOut = () => {
    wallet.signOut();
    window.location.replace(window.location.origin + window.location.pathname);
  };

  return (
    <main>
      <header>
        <h1>Name hub</h1>
        { <label>currentUser</label>
          ? <button onClick={signOut}>Log out</button>
          : <button onClick={signIn}>Log in</button>
        }
      </header>
      { currentUser
        ? <Form onSubmit={onSubmit} currentUser={currentUser} />
        : <SignIn/>
      }
      { !!currentUser && !!lots.length && <Lots lots={lots}/> }
    </main>
  );
};

App.propTypes = {
  contract: PropTypes.shape({
    profile_get: PropTypes.func.isRequired,
    lot_list: PropTypes.func.isRequired,
    lot_offer: PropTypes.func.isRequired,
  }).isRequired,
  currentUser: PropTypes.shape({
    accountId: PropTypes.string.isRequired,
    balance: PropTypes.string.isRequired
  }),
  nearConfig: PropTypes.shape({
    contractName: PropTypes.string.isRequired
  }).isRequired,
  wallet: PropTypes.shape({
    requestSignIn: PropTypes.func.isRequired,
    signOut: PropTypes.func.isRequired
  }).isRequired
};

export default App;
