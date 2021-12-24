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

  const bidPrice = useRef(null);

  const lot = props.lot;

  const bid = props.bid;

  const isNotSeller = props.currentUser !== lot.seller_id;

  const bids = props.bids;

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
        <span className="countdown"><AccessTimeFilledIcon className="icon"/><Countdown daysInHours={true} date={getCountdownTime(lot)}/></span>
      </div>
      {(isNotSeller && props.currentUser) && <div className="bid_price">
        <span className="buy-now_price">Buy now: <strong className="near-icon">{getBuyNowPrice(lot)}</strong></span>
        <button name="buy_now" onClick={(e) => bid(e, lot, getBuyNowPrice(lot))}>Buy now</button>
        <input type="number" name="bid_input" className="large" ref={bidPrice} min={getNextBidAmount(lot)} defaultValue={getNextBidAmount(lot)}/>
        <button name="bid" onClick={(e) => bid(e, lot, bidPrice.current.value)}>Bid</button>
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