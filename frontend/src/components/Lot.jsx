import React, { useRef, useState, useEffect } from 'react';
import {nearTo} from "../utils";

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

/*
LotStatus {
  OnSale,
    Withdrawn, // claim
    SaleSuccess, // last bidder
    SaleFailure, // withdrawn
}
*/

function Lot(props) {

  const lot = props.lot;

  const isNotSeller = props.currentUser !== lot.seller_id;

  const bidPrice = useRef(null);

  return (
    <li className='lot_item'>
      {console.log(lot)}
      <ul>
        <li>Lot name: <strong>{lot.lot_id}</strong></li>
        <li>Seller name: <strong>{lot.seller_id}</strong></li>
        <li>Current price: <strong>{getCurrentPrice(lot)}</strong></li>
        <li>Buy now price: <strong>{getBuyNowPrice(lot)}</strong></li>
        <li>Lot expired: <strong>{getExpiredDate(lot)}</strong></li>
      </ul>
      { props.currentUser && (isNotSeller && lot.is_active ? <div className="button_wrapper">
        <button name="buy_now" onClick={(e) => props.bid(lot, e)}>Buy now</button>
        <button name="bid" onClick={(e) => props.bid(lot, e, bidPrice.current.value)}>Bid</button>
        <input type="number" id="bid_price" ref={bidPrice} defaultValue={getNextBidAmount(lot)}/>
      </div> : isNotSeller && !lot.is_active ?
      <div className="button_wrapper">
        <button name="withdraw" onClick={(e) => props.claim(lot, e)}>Claim</button>
      </div> : !lot.is_withdrawn &&
      <div className="button_wrapper">
        <button name="withdraw" onClick={(e) => props.withdraw(lot, e)}>Withdraw</button>
      </div>)
      }
    </li>
  );
}

export default Lot;
