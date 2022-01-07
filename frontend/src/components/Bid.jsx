import React, {useRef, useState} from 'react';
import {
  getBuyNowPrice,
  getCountdownTime,
  getNextBidAmount,
  renderName,
} from "../utils";
import {AccountCircle} from "@mui/icons-material";
import Bids from "./Bids";
import {Box, IconButton, Modal} from "@mui/material";
import CloseIcon from '@mui/icons-material/Close';
import AccessTimeFilledIcon from "@mui/icons-material/AccessTimeFilled";
import Countdown from "react-countdown";

function ModalBid (props) {

  const [showBidList, setShowBidList] = useState(false);
  const [bidButtonDisabled, setBidButtonDisabled] = useState(false);
  const [bidPriceError, setBidPriceError] = useState(false);

  const bidPrice = useRef(null);

  const lot = props.lot;
  const bid = props.bid;
  const accountId = props.signedAccount;
  const isNotSeller = accountId !== lot.seller_id;

  const bids = props.bids;

  const defaultValue = Math.round(parseFloat(getNextBidAmount(lot))*100 + 1)/100;

  const checkBid = () => {
    if (bidPrice.current.value && getNextBidAmount(lot) >= bidPrice.current.value) {
      setBidPriceError(true);
      setBidButtonDisabled(true);
    } else {
      setBidPriceError(false);
      setBidButtonDisabled(false);
    }
  }

  const openBidList = (e) => {
    setShowBidList(!showBidList);
  };

  return (
    <Modal onClose={props.onClose} open={props.open}>
      <Box className="modal-container bid_modal">
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
      <div className="bid_info">
        <span className="lot_name">{renderName(lot.lot_id)}</span>
        <span className="seller_name"><AccountCircle className="icon"/>{renderName(lot.seller_id)}</span>
        <span className="countdown"><AccessTimeFilledIcon className="icon"/><Countdown date={getCountdownTime(lot)}/></span>
      </div>
      {(isNotSeller && accountId && !lot.notSafe) && <div className="bid_price">
        <span className="buy-now_price">Buy now: <strong className="near-icon">{getBuyNowPrice(lot)}</strong></span>
        <button name="buy_now" onClick={(e) => bid(e, lot, getBuyNowPrice(lot))}>Buy now</button>
        <input type="number" name="bid_input" className="large" onChange={checkBid} ref={bidPrice}
               placeholder={'min: ' + defaultValue} step={0.01} min={defaultValue} defaultValue={defaultValue}/>
        <button name="bid" onClick={(e) => bid(e, lot, bidPrice.current.value)} disabled={bidButtonDisabled}>Bid</button>
        {bidPriceError && <span className="error-input">bid value should more than: {getNextBidAmount(lot)}</span>}
      </div>}
      <div className="bid_list">
        {bids.length ? <a className="button-link" onClick={(e) => openBidList(e)}>{showBidList ? 'Hide' : 'Show'} bid list</a> : ''}
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