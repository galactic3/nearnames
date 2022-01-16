import React, {useEffect, useRef, useState} from 'react';
import ls from 'local-storage';
import { customRequestSigninFullAccess, toNear, nearTo, MIN_RESERVE_PRICE } from '../utils.js';
import {Box, FormControl, FormHelperText, IconButton, InputLabel, MenuItem, Modal, Select} from "@mui/material";
import { useForm } from "react-hook-form";
import CloseIcon from "@mui/icons-material/Close";
import ModalAlert from "./Alert";

import KeyboardArrowDownRoundedIcon from '@mui/icons-material/KeyboardArrowDownRounded';
import useConfirm from "../Hooks/useConfirm";
import Alert from "@mui/material/Alert";

function Offer (props) {

  const accountSuffix = '.' + 'testnet';
  const [alertShow, setAlertShow] = useState(false);
  const [alertContent, setAlertContent] = useState('');
  const [offerButtonDisabled, setOfferButtonDisabled] = useState(false);
  const [sameAccountError, setSameAccountError] = useState(false);
  const [priceError, setPriceError] = useState(false);
  const [buyPriceError, setBuyPriceError] = useState(false);
  const [lotId, setLotId] = useState('');
  const [priceCompareError, setPriceCompareError] = useState(false);
  const [duration, setDuration] = useState(24);
  const {register, formState: { errors }, handleSubmit} = useForm();
  const { isConfirmed } = useConfirm();

  const lotRef = useRef(null);
  const sellerRef = useRef(null);
  const priceRef = useRef(null);
  const buyPriceRef = useRef(null);

  const handleDurationChange = (event) => {
    setDuration(event.target.value);
  }

  const alertOpen = (text) => {
    setAlertShow(true);
    setAlertContent(text);
  };

  const alertHide = () => {
    setAlertShow(false);
  };

  const onBlur = (e) => {
    setLotId(e.target.value);
  }

  const checkValidationOffer = () => {
    if (!sameAccountError && !priceError && !buyPriceError && !priceCompareError) {
      setOfferButtonDisabled(false);
    } else {
      setOfferButtonDisabled(true);
    }
  }

  useEffect(() => {
    checkValidationOffer();
  }, [sameAccountError, priceError, buyPriceError, priceCompareError]);

  const checkAccounts = async () => {
    if (lotRef.current.value && sellerRef.current.value) {
      setSameAccountError(lotRef.current.value === sellerRef.current.value);
    }
  }

  const checkPrice = async () => {
    priceRef.current.value && setPriceError(toNear(priceRef.current.value).cmp(toNear(MIN_RESERVE_PRICE)) < 0);
    buyPriceRef.current.value && setBuyPriceError(toNear(buyPriceRef.current.value).cmp(toNear(MIN_RESERVE_PRICE)) < 0)
    priceRef.current.value && buyPriceRef.current.value && setPriceCompareError(toNear(buyPriceRef.current.value).cmp(toNear(priceRef.current.value)) < 0)
  }

  const onSubmit = async (e) => {
    e.preventDefault();
    setOfferButtonDisabled(true);

    const { fieldset, lot_id, seller_id, reserve_price, buy_now_price } = e.target.elements;

    if (props.signedAccount) {
      props.wallet.signOut()
    }

    const lot_account_id = lot_id.value.endsWith(accountSuffix) ? lot_id.value.trim() : lot_id.value.trim() + accountSuffix;
    const seller_account_id = seller_id.value.endsWith(accountSuffix) ? seller_id.value.trim() : seller_id.value.trim()  + accountSuffix;

    fieldset.disabled = true;

    console.log('lot check');

    const account = await props.near.account(lot_account_id);
    let balance = null;
    try {
      balance = nearTo((await account.getAccountBalance()).total);
    } catch (e) {
      alertOpen('Account ' + lot_account_id + ' not exist - you have to create it first');
      setOfferButtonDisabled(false);
      fieldset.disabled = false;
      throw console.error('Account ' + lot_account_id + ' not exist - you have to create it first')
    }

    if (balance < MIN_RESERVE_PRICE) {
      alertOpen(`Not enough balance - should be at least ${MIN_RESERVE_PRICE} NEAR available`);
      setOfferButtonDisabled(false);
      fieldset.disabled = false;
      throw console.error(`Not enough balance - should be at least ${MIN_RESERVE_PRICE} NEAR available`)
    }

    if (balance > 50 || balance > reserve_price.value) {
      let msg = `You're about to sell account ${lot_account_id} with amount bigger than reserve price on it (${balance} NEAR)`;
      if (balance > 50) {
        msg = `You're about to sell account ${lot_account_id} with signification amount on it (${balance} NEAR)`;
      }
      const confirmed = await isConfirmed(msg + `You might want to withdraw balance before offer, leaving only amount required for storage (${MIN_RESERVE_PRICE} NEAR). Do you want to proceed anyway?`);
      if (!confirmed) {
        setOfferButtonDisabled(false);
        fieldset.disabled = false;
        return;
      }
    }

    console.log('seller check');

    const seller = await props.near.account(seller_account_id);

    try {
      nearTo((await seller.getAccountBalance()).total);
    } catch (e) {
      alertOpen('Account: ' + seller_account_id + ' not exist - you have to create it first');
      setOfferButtonDisabled(false);
      fieldset.disabled = false;
      throw console.error('Account: ' + seller_account_id + ' not exist - you have to create it first')
    }

    let offerData = {
      seller_id: seller_account_id,
      reserve_price: toNear(reserve_price.value).toFixed(),
      buy_now_price: toNear(buy_now_price.value).toFixed(),
      duration: (duration * 60 * 60 * 1_000_000_000).toFixed()
    };

    const accessKeys = await account.getAccessKeys();

    ls.set(props.lsPrevKeys, accessKeys);
    ls.set(props.lsLotAccountId, lot_account_id);

    ls.set(props.nearConfig.contractName + ':lotOffer: ' + lot_account_id,
      JSON.stringify(offerData));

    // adding random Full Access Key

    await customRequestSigninFullAccess(
      props.wallet,
      props.nearConfig.contractName,
      window.location.origin + window.location.pathname + '#/offerProcess',
      window.location.origin + window.location.pathname + '#/lots'
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
          <fieldset id="fieldset">
            <div className='form-group'>
              <label htmlFor="lot_id">Lot account:</label>
              <div className="input-wrapper">
                <input
                  className="name"
                  autoComplete="off"
                  autoCapitalize="off"
                  onChange={checkAccounts}
                  onBlur={(e) => onBlur(e)}
                  type="text"
                  ref={lotRef}
                  id="lot_id"
                  required
                /><span>{accountSuffix}</span>
              </div>
              {sameAccountError && <span className="error-input">The accounts should be different</span>}
              <span className="error-input">{errors.lotId?.type === 'required' && "Seller account is required"}</span>
            </div>
            <div className='form-group'>
              <label htmlFor="seller_id">Seller account:</label>
              <div className="input-wrapper">
                <input
                  className="name"
                  autoComplete="off"
                  autoCapitalize="off"
                  onChange={checkAccounts}
                  type="text"
                  ref={sellerRef}
                  id="seller_id"
                  required
                /><span>{accountSuffix}</span>
              </div>
              <span className="error-input">{errors.sellerId?.type === 'required' && "Seller account is required"}</span>
            </div>
            {lotId && <Alert className="alert-container" severity="warning">Please SELECT <b>{lotId}</b> in wallet! Failure to choose correct account may lead to loss of funds.</Alert>}
            <div className='form-group'>
              <label htmlFor="reserve_price">Min price:</label>
              <div className="input-wrapper">
                <input
                  className="price"
                  autoComplete="off"
                  defaultValue={MIN_RESERVE_PRICE}
                  onChange={checkPrice}
                  id="reserve_price"
                  ref={priceRef}
                  min={MIN_RESERVE_PRICE}
                  step="0.01"
                  type="number"
                  required
                /><span>Near</span>
              </div>
              {priceCompareError && !priceError && !buyPriceError && <span className="error-input">Buy now price must be more then reserve</span>}
              {priceError && <span className="error-input">Min price should be more than {MIN_RESERVE_PRICE}</span>}
              <span className="error-input">{errors.price?.type === 'required' && "Price is required"}</span>
            </div>
            <div className='form-group'>
              <label htmlFor="buy_now_price">Buy now price:</label>
              <div className="input-wrapper">
              <input
                className="price"
                autoComplete="off"
                onChange={checkPrice}
                id="buy_now_price"
                ref={buyPriceRef}
                min={MIN_RESERVE_PRICE}
                step="0.01"
                type="number"
                required
              /><span>Near</span>
              </div>
              {buyPriceError && <span className="error-input">Buy price should be more than {MIN_RESERVE_PRICE}</span>}
            </div>
            <div className='form-group'>
              <label htmlFor="duration-select">Duration:</label>
              <div className="input-wrapper">
                <Select
                  labelId="duration-select-label"
                  id="duration-select"
                  value={duration}
                  onChange={handleDurationChange}
                  IconComponent={KeyboardArrowDownRoundedIcon}
                  required
                >
                  <MenuItem value={24}>1 day</MenuItem>
                  <MenuItem value={3 * 24}>3 days</MenuItem>
                  <MenuItem value={5 * 24}>5 days</MenuItem>
                  <MenuItem value={7 * 24}>7 days</MenuItem>
                  <MenuItem value={10 * 24}>10 days</MenuItem>
                  <MenuItem value={30 * 24}>30 days</MenuItem>
                </Select>
              </div>
            </div>
            <div className='form-group confirmation'>
              <label>
                <p>To ensure that buyer will receive control over the account after the sale, we require lot account to give control over itself to the marketplace contract. After full access is given, UI:</p>
                <ul className="default">
                  <li>deploys lock contract to lot account</li>
                  <li>configures marketplace account as owner</li>
                  <li>calls lot_offer to put account on sale</li>
                  <li>removes all access keys from the account</li>
                </ul>
                <p>After that moment, lot can be only unlocked by call from marketplace account.</p>
              </label>
              <div className="input-checkbox">
                <input
                  id="confirm"
                  type="checkbox"
                  required
                /><label htmlFor="confirm">Yes, I understand</label>
              </div>
              <span className="error-input">{errors.confirm?.type === 'required' && "Please apply checkbox"}</span>
            </div>
          </fieldset>
          <div className="button_wrapper">
            <button disabled={offerButtonDisabled} type="submit" className="full-width">
              Create offer
            </button>
          </div>
        </form>
        <ModalAlert
          open={alertShow}
          content={alertContent}
          onClose={() => alertHide()}/>
      </Box>
    </Modal>
  )
}

export default Offer;
