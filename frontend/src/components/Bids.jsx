import React from 'react';
import {nearTo} from "../utils";

function Bids(props) {
  return (
    <p>{props.bid.bidder_id}: <strong>{nearTo(props.bid.amount)}</strong> <span title="NEAR Tokens">Ⓝ</span></p>
  )
}

export default Bids;

