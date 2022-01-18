import React from 'react';
import {nearToCeil, renderName} from "../utils";
import AccountCircleIcon from "@mui/icons-material/AccountCircle";

function Bids(props) {
  return (
    <li><span className="bidder_name"><AccountCircleIcon className="icon"/>{renderName(props.bid.bidder_id)}</span> <strong className="near-icon">{nearToCeil(props.bid.amount)}</strong></li>
  )
}

export default Bids;

