import React, { useState } from 'react';
import ls from 'local-storage';
import { customRequestSigninFullAccess, toNear, nearTo } from '../utils.js';
import {Box, IconButton, Modal} from "@mui/material";
import CloseIcon from "@mui/icons-material/Close";

function Offer (props) {

  const accountSuffix = '.' + props.app.accountSuffix;
  const [showAlert, setShowAlert] = useState(false);
  const [contentAlert, setContentAlert] = useState('');
  const [offerButtonEnabled, setOfferButtonEnabled] = useState(true);

  const onSubmit = async (e) => {
    e.preventDefault();
    setOfferButtonEnabled(false);

    const { fieldset, lot_id, seller_id, reserve_price, buy_now_price, duration } = e.target.elements;

    if (props.signedIn) {
      props.app.wallet.signOut()
    }

    const lot_account_id = lot_id.value.endsWith(accountSuffix) ? lot_id.value : lot_id.value + accountSuffix;
    const seller_account_id = seller_id.value.endsWith(accountSuffix) ? seller_id.value : seller_id.value  + accountSuffix;

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
      seller_id: seller_account_id,
      reserve_price: toNear(reserve_price.value),
      buy_now_price: toNear(buy_now_price.value),
      duration: (duration.value * 3600000000000).toFixed()
    };

    const accessKeys = await account.getAccessKeys();

    ls.set(props.app.lsPrevKeys, accessKeys);
    ls.set(props.app.lsLotAccountId, lot_account_id);

    ls.set(props.app.config.contractName + ':lotOffer: ' + lot_account_id,
      JSON.stringify(offerData));

    // adding random Full Access Key

    await customRequestSigninFullAccess(
      props.app.wallet,
      props.app.config.contractName,
      window.location.origin + window.location.pathname + '/#/offerProcess',
      window.location.origin + window.location.pathname + + '/#/lots'
    )
  };

  return (
    <Modal onClose={props.onClose} open={props.open}>
      <Box className="modal-container offer_modal">
        <IconButton
          aria-label="close"
          onClick={props.onClose}
          className="button-icon"
          sx={{
            position: 'absolute',
            right: 8,
            top: 8,
            color: 'var(--gray)',
          }}
        >
          <CloseIcon />
        </IconButton>
        <h3 className="title">Create offer</h3>
        <form className="form_offer" onSubmit={onSubmit}>
          <fieldset>
            <div className='form-group'>
              <label htmlFor="lot_id">Lot account:</label>
              <div className="input-wrapper">
                <input
                  className="name"
                  autoComplete="off"
                  type="text"
                  id="lot_id"
                  required
                /><span>{accountSuffix}</span>
              </div>
            </div>
            <div className='form-group'>
              <label htmlFor="seller_id">Seller account:</label>
              <div className="input-wrapper">
                <input
                  className="name"
                  autoComplete="off"
                  type="text"
                  id="seller_id"
                  required
                /><span>{accountSuffix}</span>
              </div>
            </div>
            <div className='form-group'>
              <label htmlFor="reserve_price">Min price:</label>
              <div className="input-wrapper">
                <input
                  className="price"
                  autoComplete="off"
                  defaultValue="1.5"
                  id="reserve_price"
                  min="1.5"
                  step="0.01"
                  type="number"
                  required
                /><span>Near</span>
              </div>
            </div>
            <div className='form-group'>
              <label htmlFor="buy_now_price">Buy now price:</label>
              <div className="input-wrapper">
              <input
                className="price"
                autoComplete="off"
                id="buy_now_price"
                min="1.5"
                step="0.01"
                type="number"
                required
              /><span>Near</span>
              </div>
            </div>
            <div className='form-group'>
              <label htmlFor="buy_now_price">Duration:</label>
              <div className="input-wrapper">
              <input
                className="duration"
                autoComplete="off"
                id="duration"
                type="text"
              /><span>hours</span>
              </div>
            </div>
          </fieldset>
          <div className="button_wrapper">
            <button disabled={!offerButtonEnabled} type="submit" className="full-width">
              Create offer
            </button>
          </div>
        </form>
      </Box>
    </Modal>
  )
}

export default Offer;