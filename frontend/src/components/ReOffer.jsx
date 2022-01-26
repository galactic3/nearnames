import React, {useEffect, useRef, useState} from 'react';
import {
  toNear,
  nearToFloor,
  MIN_RESERVE_PRICE, renderName,
} from '../utils.js';
import {Box, FormControl, FormHelperText, IconButton, InputLabel, MenuItem, Modal, Select} from "@mui/material";
import { useForm } from "react-hook-form";
import CloseIcon from "@mui/icons-material/Close";

import KeyboardArrowDownRoundedIcon from '@mui/icons-material/KeyboardArrowDownRounded';
import useConfirm from "../Hooks/useConfirm";
import Loader from "./Loader";
import Alert from "@mui/material/Alert";

function Offer (props) {

  const [offerButtonDisabled, setOfferButtonDisabled] = useState(false);
  const [priceError, setPriceError] = useState(false);
  const [buyPriceError, setBuyPriceError] = useState(false);
  const [priceCompareError, setPriceCompareError] = useState(false);
  const [showSuccess, setShowSuccess] = useState(false);
  const [showLoader, setShowLoader] = useState(false);
  const [duration, setDuration] = useState(24);
  const {register, formState: { errors }, handleSubmit} = useForm();

  const priceRef = useRef(null);
  const buyPriceRef = useRef(null);

  const lot_id = props.lot && props.lot.lot_id;

  const handleDurationChange = (event) => {
    setDuration(event.target.value);
  }

  const checkValidationOffer = () => {
    if (!priceError && !buyPriceError && !priceCompareError) {
      setOfferButtonDisabled(false);
    } else {
      setOfferButtonDisabled(true);
    }
  }

  useEffect(() => {
    checkValidationOffer();
  }, [priceError, buyPriceError, priceCompareError]);

  useEffect(() => {
    setPriceError(false);
    setBuyPriceError(false);
    setPriceCompareError(false);
  }, [props]);

  const checkPrice = async () => {
    priceRef.current.value && setPriceError(toNear(priceRef.current.value).cmp(toNear(MIN_RESERVE_PRICE)) < 0);
    buyPriceRef.current.value && setBuyPriceError(toNear(buyPriceRef.current.value).cmp(toNear(MIN_RESERVE_PRICE)) < 0)
    priceRef.current.value && buyPriceRef.current.value && setPriceCompareError(toNear(buyPriceRef.current.value).cmp(toNear(priceRef.current.value)) < 0)
  }

  const onSubmit = async (e) => {
    e.preventDefault();
    setOfferButtonDisabled(true);

    const { fieldset, reserve_price, buy_now_price } = e.target.elements;

    fieldset.disabled = true;

    let offerData = {
      lot_id: lot_id,
      reserve_price: toNear(reserve_price.value).toFixed(),
      buy_now_price: toNear(buy_now_price.value).toFixed(),
      duration: (duration * 60 * 60 * 1_000_000_000).toFixed()
    };

    setShowLoader(true);
    await props.contract.lot_reoffer(offerData);
    setShowLoader(false);
    setShowSuccess(true);
    setOfferButtonDisabled(false);

  };

  const onClose = () => {
    if (showLoader) {
      return;
    }
    props.onClose(showSuccess);
    setShowSuccess(false);
  }

  return (
    <Modal onClose={() => onClose()} open={props.open}>
      <Box className="modal-container offer_modal">
        <IconButton
          aria-label="close"
          onClick={() => onClose()}
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
        <h3 className="title">Re-offer <strong>{renderName(lot_id)}</strong></h3>
        { showLoader ? <Loader/> : showSuccess ? <Alert className="alert-container" severity="success">Success! Account ${lot_id} is now on sale.</Alert> :
          <form className="form_offer" onSubmit={onSubmit}>
            <fieldset id="fieldset">
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
                  <p>The app takes 10% fee of all rewards paid to the seller. <a href="https://github.com/galactic3/nearnames/wiki/Money-flow" target="_blank">Read more</a></p>
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
        }
      </Box>
    </Modal>
  )
}

export default Offer;
