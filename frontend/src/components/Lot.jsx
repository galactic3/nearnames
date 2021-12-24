import React, { useRef, useState, useEffect } from 'react';
import {getBuyNowPrice, getCountdownTime, getCurrentPrice, nearTo, renderName, tsNear2JS} from "../utils";
import AccountCircleIcon from '@mui/icons-material/AccountCircle';
import Countdown from "react-countdown";
import AccessTimeFilledIcon from '@mui/icons-material/AccessTimeFilled';

function getLastBidder(bids) {
  return bids.length ? bids[bids.length-1].bidder_id : '';
}

function Lot(props) {

  const [bids, setBids] = useState([]);

  const lot = props.lot;
  const contract = props.contract;
  const isNotSeller = props.currentUser !== lot.seller_id;
  const isLastBidder = props.currentUser === getLastBidder(bids);

  useEffect(() => {
    contract.lot_bid_list({'lot_id': lot.lot_id}).then(setBids);
  }, []);

  const renderButton = (lot) => {
    switch(lot.status) {
      case 'OnSale':
        return (
          <div className="button_wrapper">
            <button name="bid" className="outlined" onClick={(e) => props.openBid(lot, bids, e)}>{isNotSeller && props.currentUser ? 'Buy or bid' : 'Show bid list'}</button>
          </div>)
      case 'SaleSuccess':
        return (isLastBidder && <div className="button_wrapper">
          <button name="claim" className="outlined" onClick={(e) => props.claim(lot, e)}>Claim</button>
        </div>)
      case 'SaleFailure':
        return (!isNotSeller && <div className="button_wrapper">
          <button name="withdraw" className="outlined" disabled={props.loader} onClick={(e) => props.withdraw(lot, e)}>{props.loader ? 'Loading...' : 'Withdraw'}</button>
        </div>)
      case 'Withdrawn':
        return (!isNotSeller && <div className="button_wrapper">
          <button name="claim_back" className="outlined" onClick={(e) => props.claim(lot, e)}>Claim</button>
        </div>)
    }
  }

  return (
    <li className='lot_item'>
      <div className="lot_info">
        <span className="lot_name">{renderName(lot.lot_id)}</span>
        <span className="seller_name"><AccountCircleIcon className="icon"/>{renderName(lot.seller_id)}</span>
      </div>
      {props.showStatus && <div className="lot_status">
        <span className={'badge ' + lot.status}>{lot.status}</span>
      </div>}
      <div className="lot_price">
        <span className="current_price near-icon">{getCurrentPrice(lot)}</span>
        <span className="buy-now_price">Buy now: <strong className="near-icon">{getBuyNowPrice(lot)}</strong></span>
      </div>
      <div className="lot_action">
        {renderButton(lot)}
        {getCountdownTime(lot) > Date.now() && <span className="countdown"><AccessTimeFilledIcon className="icon"/><Countdown daysInHours={true} date={getCountdownTime(lot)}/></span>}
      </div>
    </li>
  );
}

export default Lot;
