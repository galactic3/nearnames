import React, { useRef, useState, useEffect } from 'react';
import { nearTo } from "../utils";
import Bids from "./Bids";
import {Spinner} from "react-bootstrap";

function getReservePrice(lot) {
  return nearTo(lot.reserve_price, 2);
}

function getExpiredDate(lot) {
  return new Date(Math.floor(lot.finish_timestamp/1000000)).toUTCString();
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

  const lot = props.lot;

  const contract = props.contract;

  const isNotSeller = props.currentUser !== lot.seller_id;

  const bidPrice = useRef(null);

  const [bids, setBids] = useState([]);

  const [showBidList, setShowBidList] = useState(false);

  const isLastBidder = props.currentUser === getLastBidder(bids);

  useEffect(() => {
    contract.lot_bid_list({'lot_id': lot.lot_id}).then(setBids);
  }, []);


  const renderButton = (lot) => {
    switch(lot.status) {
      case 'OnSale':
        return (isNotSeller && <div className="button_wrapper">
          <button name="buy_now" onClick={(e) => props.bid(lot, e, getBuyNowPrice(lot))}>Buy now</button>
          <button name="bid" onClick={(e) => props.bid(lot, e, bidPrice.current.value)}>Bid</button>
          <input type="number" name="bid_price" className="large" ref={bidPrice} defaultValue={getNextBidAmount(lot)}/>
        </div>)
      case 'SaleSuccess':
        return (isLastBidder && <div className="button_wrapper">
          <button name="claim" onClick={(e) => props.claim(lot, e)}>Claim</button>
        </div>)
      case 'SaleFailure':
        return (!isNotSeller && <div className="button_wrapper">
          {props.loader ? <Spinner animation="border" /> : <button name="withdraw" onClick={(e) => props.withdraw(lot, e)}>Withdraw</button>}
        </div>)
      case 'Withdrawn':
        return (!isNotSeller && <div className="button_wrapper">
          <button name="claim_back" onClick={(e) => props.claim(lot, e)}>Claim back</button>
        </div>)
    }
  }

  return (
    <li className='lot_item'>
      <ul>
        <li>Lot name: <strong>{lot.lot_id}</strong></li>
        <li>Seller name: <strong>{lot.seller_id}</strong></li>
        <li>Current price: <strong>{getCurrentPrice(lot)}</strong></li>
        <li>Buy now price: <strong>{getBuyNowPrice(lot)}</strong></li>
        {bids.length ? <li>Last bidder name: <strong>{getLastBidder(bids)}</strong></li> : ''}
        <li>Lot status: <strong>{lot.status}</strong></li>
      </ul>
      {props.currentUser && renderButton(lot)}
      {bids.length ? <a className="button-link" onClick={(e) => setShowBidList(!showBidList)}>{showBidList ? 'Hide' : 'Show'} bid list</a> : ''}
      {showBidList ? (<div className="bids_list">
        {bids.map((bid, i) =>
          <Bids key={i} bid={bid}/>
        )}
      </div>) : ''}
    </li>
  );
}

export default Lot;
