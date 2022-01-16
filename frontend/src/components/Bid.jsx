import React, {useEffect, useRef, useState} from 'react';
import {
  getBuyNowPrice,
  getCountdownTime,
  getNextBidAmount,
  renderName,
  toNear,
} from "../utils";
import {AccountCircle} from "@mui/icons-material";
import Bids from "./Bids";
import {Box, IconButton, Modal} from "@mui/material";
import CloseIcon from '@mui/icons-material/Close';
import AccessTimeFilledIcon from "@mui/icons-material/AccessTimeFilled";
import Countdown from "react-countdown";

function ModalBid (props) {

  const lot = props.lot;
  const [value, setValue] = useState(getNextBidAmount(lot));
  const [defaultValue, setDefaultValue] = useState(getNextBidAmount(lot));
  const [showBidList, setShowBidList] = useState(false);
  const [bidButtonDisabled, setBidButtonDisabled] = useState(false);
  const [bidPriceError, setBidPriceError] = useState(false);

  const bidPrice = useRef(null);

  const bid = props.bid;
  const contract = props.contract;
  const accountId = props.signedAccount;
  const isNotSeller = accountId !== lot.seller_id;

  const [bids, setBids] = useState([]);

  useEffect(() => {
    setShowBidList(false);
    setBidPriceError(false);
    setBidButtonDisabled(false);
    setDefaultValue(getNextBidAmount(props.lot));
    setValue(getNextBidAmount(props.lot));
    setBids([]);
  }, [props]);

  const onChangeBid = (e) => {
    const value = e.target.value;
    checkBid();
    setValue(value);
  }

  const checkBid = () => {
    if (bidPrice.current.value && toNear(defaultValue).cmp(toNear(bidPrice.current.value)) > 0) {
      setBidPriceError(true);
      setBidButtonDisabled(true);
    } else {
      setBidPriceError(false);
      setBidButtonDisabled(false);
    }
  }

  const getBidList = async (lot) => {
    contract.lot_bid_list({'lot_id': lot.lot_id}).then(setBids);
  }

  const openBidList = async (e) => {
    if (!bids.length) {
      await getBidList(lot);
    }
    setShowBidList(!showBidList);
  };

  const clearState = () => {
    props.onClose();
  }

  return (
    <Modal open={props.open} onClose={() => clearState()}>
      <Box className="modal-container bid_modal">
      <IconButton
        aria-label="close"
        onClick={() => clearState()}
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
      <div className="bid_info">
        <span className="lot_name">{renderName(lot.lot_id)}</span>
        <span className="seller_name"><AccountCircle className="icon"/>{renderName(lot.seller_id)}</span>
        {lot.status === 'OnSale' && <span className="countdown"><AccessTimeFilledIcon className="icon"/><Countdown date={getCountdownTime(lot)}/></span>}
      </div>
      {(isNotSeller && accountId && !lot.notSafe && lot.status === 'OnSale') && <div className="bid_price">
        <span className="buy-now_price">Buy now: <strong className="near-icon">{getBuyNowPrice(lot)}</strong></span>
        <button name="buy_now" onClick={(e) => bid(e, lot.lot_id, getBuyNowPrice(lot))}>Buy now</button>
        <input type="number" name="bid_input" className="large" onChange={(e) => onChangeBid(e)} ref={bidPrice}
               placeholder={'min: ' + defaultValue} step="0.01" min={defaultValue} value={value}/>
        <button name="bid" onClick={(e) => bid(e, lot.lot_id, bidPrice.current.value)} disabled={bidButtonDisabled}>Bid</button>
        {bidPriceError && <span className="error-input">bid value should more than: {defaultValue}</span>}
      </div>}
      {lot.notSafe && <div className="lot_status">
        <span className='badge'>Not safe</span>
      </div>}
      <div className="bid_list">
        {lot.last_bidder_id ? <a className="button-link" onClick={(e) => openBidList(e)}>{showBidList ? 'Hide' : 'Show'} bid list</a> : ''}
        {showBidList ? (<ul className="bids_list">
          {bids.map((bid, i) =>
            <Bids key={i} bid={bid}/>
          )}
        </ul>) : ''}
      </div>
      </Box>
    </Modal>
  )
}

export default ModalBid;