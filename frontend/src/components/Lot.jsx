import React, { useRef, useState, useEffect } from 'react';
import {nearTo, renderName, tsNear2JS} from "../utils";
import {Spinner} from "react-bootstrap";
import AccountCircleIcon from '@mui/icons-material/AccountCircle';
import Countdown from "react-countdown";
import AccessTimeFilledIcon from '@mui/icons-material/AccessTimeFilled';

function getReservePrice(lot) {
  return nearTo(lot.reserve_price, 2);
}

function getCountdownTime(lot) {
  return new Date(tsNear2JS(lot.finish_timestamp)).getTime();
}

function getCurrentPrice(lot) {
  return lot.last_bid_amount ? nearTo(lot.last_bid_amount, 2) : getReservePrice(lot);
}

function getNextBidAmount(lot) {
  return lot.next_bid_amount ? nearTo(lot.next_bid_amount, 2) : getReservePrice(lot);
}

function getBuyNowPrice(lot) {
  return lot.buy_now_price ? nearTo(lot.buy_now_price, 2) : getReservePrice(lot);
}

function getLastBidder(bids) {
  return bids.length ? bids[bids.length-1].bidder_id : '';
}

function Lot(props) {

  const [bids, setBids] = useState([]);

  const lot = props.lot;
  const contract = props.contract;
  const isNotSeller = props.currentUser !== lot.seller_id;
  const bidPrice = useRef(null);
  const isLastBidder = props.currentUser === getLastBidder(bids);

  useEffect(() => {
    contract.lot_bid_list({'lot_id': lot.lot_id}).then(setBids);
  }, []);

  const renderButton = (lot) => {
    switch(lot.status) {
      case 'OnSale':
        return (isNotSeller &&
          <div className="button_wrapper">
            <button name="bid" className="outlined" onClick={(e) => props.bid(lot, bids, e)}>Buy or bid</button>
            {/*<button name="buy_now" onClick={(e) => props.bid(lot, e, getBuyNowPrice(lot))}>Buy now</button>
            <button name="bid" onClick={(e) => props.bid(lot, e, bidPrice.current.value)}>Bid</button>
            <input type="number" name="bid_price" className="large" ref={bidPrice} defaultValue={getNextBidAmount(lot)}/>*/}
          </div>)
      case 'SaleSuccess':
        return (isLastBidder && <div className="button_wrapper">
          <button name="claim" className="outlined" onClick={(e) => props.claim(lot, e)}>Claim</button>
        </div>)
      case 'SaleFailure':
        return (!isNotSeller && <div className="button_wrapper">
          {props.loader ? <Spinner animation="border" /> : <button name="withdraw" className="outlined" onClick={(e) => props.withdraw(lot, e)}>Withdraw</button>}
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
      <div className="lot_status">
        <span className="badge">{lot.status}</span>
      </div>
      <div className="lot_price">
        <span className="current_price">{getCurrentPrice(lot)} Near</span>
        <span className="buy-now_price">Buy now: <strong>{getBuyNowPrice(lot)} Near</strong></span>
      </div>
      <div className="lot_action">
        {props.currentUser && renderButton(lot)}
        <span className="countdown"><AccessTimeFilledIcon className="icon"/><Countdown daysInHours={true} date={getCountdownTime(lot)}/></span>
      </div>
    </li>
  );
}

export default Lot;
