import React, { useState, useEffect } from 'react';
import {getBuyNowPrice, getCountdownTime, getCurrentPrice, nearTo, renderName, tsNear2JS} from "../utils";
import AccountCircleIcon from '@mui/icons-material/AccountCircle';
import Countdown from "react-countdown";
import AccessTimeFilledIcon from '@mui/icons-material/AccessTimeFilled';
import { BrowserView, MobileView, isBrowser, isMobile } from 'react-device-detect';

function getLastBidder(bids) {
  return bids.length ? bids[bids.length-1].bidder_id : '';
}

function Lot(props) {

  const [bids, setBids] = useState([]);

  const lot = props.lot;
  const contract = props.contract;
  const accountId = props.signedAccount;
  const isNotSeller = accountId !== lot.seller_id;
  const isLastBidder = accountId === getLastBidder(bids);

  useEffect(() => {
    contract.lot_bid_list({'lot_id': lot.lot_id}).then(setBids);
  }, []);

  const renderButton = (lot) => {
    switch(lot.status) {
      case 'OnSale':
        if (!isNotSeller && !lot.last_bidder_id) {
          return (<div className="button_wrapper">
            <button name="withdraw" className="outlined" onClick={(e) => props.withdraw(lot, e)}>Withdraw</button>
          </div>);
        }
        return (
          <div className="button_wrapper">
            <button name="bid" className="outlined" onClick={(e) => props.openBid(lot, bids, e)}>{isNotSeller && accountId ? 'Buy or bid' : 'Show details'}</button>
          </div>)
      case 'SaleSuccess':
        return (isLastBidder ? <div className="button_wrapper">
          <button name="claim" className="outlined" onClick={(e) => props.claim(lot, e)}>Claim</button>
        </div> : <div className="button_wrapper">
          <button name="bid" className="outlined" onClick={(e) => props.openBid(lot, bids, e)}>Show details</button>
        </div>)
      case 'SaleFailure':
        return (!isNotSeller && <div className="button_wrapper">
          <button name="withdraw" className="outlined" onClick={(e) => props.withdraw(lot, e)}>Withdraw</button>
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
        {<span className="countdown"><AccessTimeFilledIcon className="icon"/><Countdown date={getCountdownTime(lot)}/></span>}
      </div>
      {props.showStatus && <div className="lot_status">
        <span className={'badge ' + lot.status}>{lot.status}</span>
      </div>}
      {lot.notSafe && <div className="lot_status">
        <span className='badge'>Not safe</span>
      </div>}
      <div className="lot_price">
        <span className="current_price near-icon">{getCurrentPrice(lot)}</span>
        <span className="buy-now_price">Buy now: <strong className="near-icon">{getBuyNowPrice(lot)}</strong></span>
      </div>
      <div className="lot_action">
        {renderButton(lot)}
        {lot.status === 'OnSale' && getCountdownTime(lot) > Date.now() && isBrowser && <span className="countdown"><AccessTimeFilledIcon className="icon"/><Countdown date={getCountdownTime(lot)}/></span>}
      </div>
    </li>
  );
}

export default Lot;
