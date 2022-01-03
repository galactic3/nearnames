import React, {useRef, useState} from 'react';
import ls from 'local-storage';
import { customRequestSigninFullAccess, toNear, nearTo } from '../utils.js';
import {Box, FormControl, FormHelperText, IconButton, InputLabel, MenuItem, Modal, Select} from "@mui/material";
import { useForm } from "react-hook-form";
import CloseIcon from "@mui/icons-material/Close";
import ModalAlert from "./Alert";

function Offer (props) {

  const accountSuffix = '.' + 'testnet';
  const [alertShow, setAlertShow] = useState(false);
  const [alertContent, setAlertContent] = useState('');
  const [offerButtonDisabled, setOfferButtonDisabled] = useState(false);
  const [sameAccountError, setSameAccountError] = useState('');
  const [priceError, setPriceError] = useState(false);
  const [buyPriceError, setBuyPriceError] = useState(false);
  const [priceCompareError, setPriceCompareError] = useState(false);
  const [durationError, setDurationError] = useState(false);
  const [duration, setDuration] = useState('');
  const {register, formState: { errors }, handleSubmit} = useForm();

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

  const checkAccounts = async () => {
    if (lotRef.current.value && sellerRef.current.value &&
        lotRef.current.value === sellerRef.current.value) {
      setSameAccountError(true);
      setOfferButtonDisabled(true);
    } else {
      setSameAccountError(false);
      setOfferButtonDisabled(false);
    }
  }

  const checkPrice = async () => {
    if (priceRef.current.value && parseFloat(priceRef.current.value) < 1.5) {
      setPriceError(true);
      setOfferButtonDisabled(true);
    } else {
      setPriceError(false);
      setOfferButtonDisabled(false);
    }

    if (buyPriceRef.current.value && parseFloat(buyPriceRef.current.value) < 1.5) {
      setBuyPriceError(true);
      setOfferButtonDisabled(true);
    } else {
      setBuyPriceError(false);
      setOfferButtonDisabled(false);
    }
    if (buyPriceRef.current.value && priceRef.current.value >= buyPriceRef.current.value) {
      setPriceCompareError(true);
      setOfferButtonDisabled(true);
    } else {
      setPriceCompareError(false);
      setOfferButtonDisabled(false);
    }
  }

  const checkAccountExist = async (account) => {
    let balance = null;
    try {
      balance = nearTo((await account.getAccountBalance()).total);
      if (balance < 1.5) {
        alert('Not enough balance - should be at least 1.5 NEAR available');
        setOfferButtonDisabled(false);
        console.error('Not enough balance - should be at least 1.5 NEAR available')
      }
    } catch (e) {
      alert('Account not exist - you have to create it first');
      console.error('Account not exist - you have to create it first');
    }
  }

  const onSubmit = async (e) => {
    e.preventDefault();
    setOfferButtonDisabled(true);

    const { fieldset, lot_id, seller_id, reserve_price, buy_now_price } = e.target.elements;

    if (props.signedIn) {
      props.app.wallet.signOut()
    }

    const lot_account_id = lot_id.value.endsWith(accountSuffix) ? lot_id.value : lot_id.value + accountSuffix;
    const seller_account_id = seller_id.value.endsWith(accountSuffix) ? seller_id.value : seller_id.value  + accountSuffix;

    fieldset.disabled = true;

    const account = await props.app.near.account(lot_account_id);
    let balance = null;
    try {
      balance = nearTo((await account.getAccountBalance()).total);
    } catch (e) {
      alertOpen('Account ' + lot_account_id + ' not exist - you have to create it first');
      setOfferButtonDisabled(false);
      fieldset.disabled = false;
      throw console.error('Account ' + lot_account_id + ' not exist - you have to create it first')
    }

    if (balance < 1.5) {
      alertOpen('Not enough balance - should be at least 1.5 NEAR available');
      setOfferButtonDisabled(false);
      fieldset.disabled = false;
      throw console.error('Not enough balance - should be at least 1.5 NEAR available')
    }

    const seller = await props.app.near.account(seller_account_id);

    try {
      balance = nearTo((await seller.getAccountBalance()).total);
    } catch (e) {
      alertOpen('Account: ' + seller_account_id + ' not exist - you have to create it first');
      setOfferButtonDisabled(false);
      fieldset.disabled = false;
      throw console.error('Account: ' + seller_account_id + ' not exist - you have to create it first')
    }

    let offerData = {
      seller_id: seller_account_id,
      reserve_price: toNear(reserve_price.value),
      buy_now_price: toNear(buy_now_price.value),
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
          <fieldset id="fieldset">
            <div className='form-group'>
              <label htmlFor="lot_id">Lot account:</label>
              <div className="input-wrapper">
                <input
                  className="name"
                  autoComplete="off"
                  onChange={checkAccounts}
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
                  onChange={checkAccounts}
                  type="text"
                  ref={sellerRef}
                  id="seller_id"
                  required
                /><span>{accountSuffix}</span>
              </div>
              <span className="error-input">{errors.sellerId?.type === 'required' && "Seller account is required"}</span>
            </div>
            <div className='form-group'>
              <label htmlFor="reserve_price">Min price:</label>
              <div className="input-wrapper">
                <input
                  className="price"
                  autoComplete="off"
                  defaultValue="1.5"
                  onChange={checkPrice}
                  id="reserve_price"
                  ref={priceRef}
                  min="1.5"
                  step="0.01"
                  type="number"
                  required
                /><span>Near</span>
              </div>
              {priceCompareError && !priceError && !buyPriceError && <span className="error-input">Buy now price must be less then reserve</span>}
              {priceError && <span className="error-input">Min price should be more than 1.5</span>}
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
                min="1.5"
                step="0.01"
                type="number"
                required
              /><span>Near</span>
              </div>
              {buyPriceError && <span className="error-input">Buy price should be more than 1.5</span>}
            </div>
            <div className='form-group'>
              <label htmlFor="duration-select">Duration:</label>
              <div className="input-wrapper">
                <Select
                  labelId="duration-select-label"
                  id="duration-select"
                  value={duration}
                  onChange={handleDurationChange}
                  sx={{
                    width: '100%',
                    backgroundColor: 'var(--main)',
                    height: '40px',
                    color: 'var(--primary)',
                    fontSize: '14px',
                    "& .MuiOutlinedInput-notchedOutline": {
                      border: "none",
                      maxHeight: '40px',
                    },
                    "& .MuiOutlinedInput-input.MuiInputBase-input": {
                      paddingTop: '5px',
                      paddingBottom: '5px',
                    },
                    "& .MuiSelect-icon.MuiSelect-iconOutlined": {
                      color: 'var(--primary)',
                    },
                  }}
                >
                  <MenuItem value={1 * 24}>1 day</MenuItem>
                  <MenuItem value={3 * 24}>3 days</MenuItem>
                  <MenuItem value={5 * 24}>5 days</MenuItem>
                  <MenuItem value={7 * 24}>7 days</MenuItem>
                  <MenuItem value={10 * 24}>10 days</MenuItem>
                </Select>
              </div>
            </div>
            <div className='form-group confirmation'>
              <div className="input-checkbox">
                <input
                  id="confirm"
                  type="checkbox"
                  required
                />
                <label htmlFor="confirm">
                  <p>To ensure that buyer will receive control over the account after the sale, we require lot account to give control over itself to the marketplace contract. After full access is given, UI:</p>
                  <ul className="default">
                    <li>deploys lock contract to lot account</li>
                    <li>configures marketplace account as owner</li>
                    <li>calls lot_offer to put account on sale</li>
                    <li>removes all access keys from the account</li>
                  </ul>
                  <p>After that moment, lot can be only unlocked by call from marketplace account.</p>
                </label>
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
