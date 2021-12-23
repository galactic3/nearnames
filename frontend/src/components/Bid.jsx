import React, {useEffect, useState} from 'react';
import TextField from '@mui/material/TextField';
import Dialog from '@mui/material/Dialog';
import DialogContent from '@mui/material/DialogContent';
import DialogTitle from '@mui/material/DialogTitle';
import {BOATLOAD_OF_GAS, fetchBidSafety, renderName, toNear} from "../utils";
import {AccountCircle} from "@mui/icons-material";
import Bids from "./Bids";

function ModalBid (props) {

  const [showBidList, setShowBidList] = useState(false);

  const lot = props.lot;

  const bids = props.bids;

  const bid = async (lot, e, value) => {
    const isBuyNowButton = e.target.name === 'buy_now';
    let nValue = toNear(value);
    if (nValue < lot.reserve_price) {
      alert ("enter value more than: " + lot.reserve_price);
      return;
    } else if (lot.next_bid_amount && nValue < lot.next_bid_amount) {
      alert ("enter value more than: " + lot.next_bid_amount);
      return;
    }
    e.target.disabled = true;
    const bid_price = isBuyNowButton ? lot.buy_now_price : nValue;
    const { codeHash, accessKeysLen, lockerOwner } = await fetchBidSafety(lot.lot_id, props.app.near);
    const isSafe = codeHash === 'DKUq738xnns9pKjpv9GifM68UoFSmfnBYNp3hsfkkUFa' &&
      accessKeysLen === 0 &&
      lockerOwner === props.app.config.contractName;
    console.log(codeHash, accessKeysLen, lockerOwner);
    if (!isSafe) {
      alert("account is not safe");
      e.target.disabled = false;
      return;
    }
    props.contract.lot_bid({'lot_id': lot.lot_id}, BOATLOAD_OF_GAS, bid_price).then(() => {
      props.getLots();
    });
  };

  const openBidList = (e) => {
    setShowBidList(!showBidList);
  };

  return (
    <Dialog className="bid_modal" {...props}>
      <DialogContent>
        <div className="bid_info">
          <span className="lot_name">{renderName(props.lot.lot_id)}</span>
          <span className="seller_name"><AccountCircle className="icon"/>{renderName(props.lot.seller_id)}</span>
        </div>
        <div className="bid_price">
          <span>Buy now: <strong>4 Near</strong></span>
        </div>
        <div className="bid_list">
          {bids.length ? <a className="button-link" onClick={(e) => openBidList(e)}>{showBidList ? 'Hide' : 'Show'} bid list</a> : ''}
          {showBidList ? (<div className="bids_list">
            {bids.map((bid, i) =>
              <Bids key={i} bid={bid}/>
            )}
          </div>) : ''}
        </div>
        <TextField
          autoFocus
          margin="dense"
          id="name"
          label="Email Address"
          type="email"
          fullWidth
          variant="standard"
        />
      </DialogContent>
    </Dialog>
  )
}

export default ModalBid;