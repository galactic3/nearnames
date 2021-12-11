import React, { useState } from 'react';
import ls from 'local-storage';
import { customRequestSigninFullAccess, toNear, nearTo } from '../utils.js';

function OfferPage (props) {

  const accountSuffix = props.app.accountSuffix;
  const [offerButtonEnabled, setOfferButtonEnabled] = useState(true);

  const onSubmit = async (e) => {
    e.preventDefault();
    setOfferButtonEnabled(false);

    const { fieldset, lot_id, seller_id, reserve_price, buy_now_price, duration } = e.target.elements;

    if (props.signedIn) {
      props.app.logOut()
    }

    const lot_account_id = lot_id.value;

    fieldset.disabled = true;

    if (lot_account_id === seller_id) {
      alert('Accounts must be different');
      setOfferButtonEnabled(true);
      throw console.error('Accounts must be different');
    }

    const account = await props.app.near.account(lot_account_id);
    let balance = null;
    try {
      balance = nearTo((await account.getAccountBalance()).total);
    } catch (e) {
      alert('Account not exist - you have to create it first');
      setOfferButtonEnabled(true);
      fieldset.disabled = false;
      throw console.error('Account not exist - you have to create it first')
    }

    if (balance < 1.5) {
      alert('Not enough balance - should be at least 1.5 NEAR available (1.5 total usually works)');
      setOfferButtonEnabled(true);
      fieldset.disabled = false;
      throw console.error('Not enough balance - should be at least 1.5 NEAR available')
    }

    let offerData = {
      seller_id: seller_id.value + '.' + accountSuffix,
      reserve_price: toNear(reserve_price.value),
      buy_now_price: toNear(buy_now_price.value),
      duration: (duration.value * 3600000000000).toFixed()
    };

    const accessKeys = await props.app.account.getAccessKeys();

    ls.set(props.app.lsPrevKeys, accessKeys);
    ls.set(props.app.lsLotAccountId, lot_account_id);

    ls.set(props.app.config.contractName + ':lotOffer: ' + lot_account_id,
      JSON.stringify(offerData));

    // adding random Full Access Key

    await customRequestSigninFullAccess(
      props.app.wallet,
      props.app.config.contractName,
      window.location.origin + '/#/offerProcess',
      window.location.origin + '/#/offer'
    )

  };

  return (
    <div className="form_offer">
    <form onSubmit={onSubmit}>
      <fieldset id="fieldset">
        <p>
          <label htmlFor="lot_id">Lot account:</label>
          <input
            autoComplete="off"
            type="text"
            id="lot_id"
            defaultValue={props.app.accountId}
            required
          />
        </p>
        <p>
          <label htmlFor="seller_id">Seller account:</label>
          <input
            autoComplete="off"
            type="text"
            id="seller_id"
            required
          /> .{accountSuffix}
        </p>
        <p>
          <label htmlFor="reserve_price">Min price:</label>
          <input
            autoComplete="off"
            defaultValue="1.5"
            id="reserve_price"
            min="1.5"
            step="0.01"
            type="number"
            required
          />
          <span title="NEAR Tokens">Ⓝ</span>
        </p>
        <p>
          <label htmlFor="buy_now_price">Buy now price:</label>
          <input
            autoComplete="off"
            id="buy_now_price"
            min="1.5"
            step="0.01"
            type="number"
            required
          />
          <span title="NEAR Tokens">Ⓝ</span>
        </p>
        <p>
          <label htmlFor="buy_now_price">Duration:</label>
          <input
            autoComplete="off"
            id="duration"
            type="number"
          /> hours
        </p>
        <button disabled={!offerButtonEnabled} type="submit">
          Create offer
        </button>
      </fieldset>
    </form>
    </div>
  )
}

export default OfferPage;