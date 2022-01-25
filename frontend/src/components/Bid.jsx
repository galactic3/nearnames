import React, {useEffect, useRef, useState} from 'react';
import {
  getBuyNowPrice,
  getCountdownTime,
  getNextBidAmount,
  renderName,
  toNear,
} from "../utils";
import {AccountCircle} from "@mui/icons-material";
import Bids from "./Bids";
import {Box, IconButton, Modal} from "@mui/material";
import CloseIcon from '@mui/icons-material/Close';
import AccessTimeFilledIcon from "@mui/icons-material/AccessTimeFilled";
import Countdown from "react-countdown";
import Alert from "@mui/material/Alert";
import Loader from "./Loader";

function ModalBid (props) {

  const lot = props.lot;
  const [value, setValue] = useState('');
  const [defaultValue, setDefaultValue] = useState('');
  const [showBidList, setShowBidList] = useState(false);
  const [lotNameExpanded, setLotNameExpanded] = useState(false);
  const [bidButtonDisabled, setBidButtonDisabled] = useState(false);
  const [bidPriceError, setBidPriceError] = useState(false);
  const [showLoader, setShowLoader] = useState(true);

  const bidPrice = useRef(null);

  const bid = props.bid;
  const contract = props.contract;
  const accountId = props.signedAccount;
  const isNotSeller = accountId !== (lot && lot.seller_id);

  const [bids, setBids] = useState([]);

  useEffect(() => {
    setShowBidList(false);
    setBidPriceError(false);
    setLotNameExpanded(false);
    setBidButtonDisabled(false);
    setDefaultValue(lot && getNextBidAmount(lot));
    setValue(lot && getNextBidAmount(lot));
    setShowLoader(!lot && lot !== null);
    setBids([]);
  }, [props]);

  const onChangeBid = (e) => {
    const value = e.target.value;
    checkBid();
    setValue(value);
  }

  const checkBid = () => {
    if (bidPrice.current.value && toNear(defaultValue).cmp(toNear(bidPrice.current.value)) > 0) {
      setBidPriceError(true);
      setBidButtonDisabled(true);
    } else {
      setBidPriceError(false);
      setBidButtonDisabled(false);
    }
  }

  const getBidList = async (lot) => {
    contract.lot_bid_list({'lot_id': lot.lot_id}).then(setBids);
  }

  const openBidList = async (e) => {
    if (!bids.length) {
      try {
        await getBidList(lot);
      } catch (e) {
        console.error(e);
      }
    }
    setShowBidList(!showBidList);
  };

  const onClose = () => {
    props.onClose();
  }

  return (
    <Modal open={props.open} onClose={() => onClose()}>
      <Box className="modal-container bid_modal">
      <IconButton
        aria-label="close"
        onClick={() => onClose()}
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
        { showLoader ? <Loader/> : lot ?
          <div>
            <div className="bid_info">
              <span className={'lot_name' + (lotNameExpanded ? ' expand' : '')} onClick={() => setLotNameExpanded(!lotNameExpanded)}>{renderName(lot.lot_id)}</span>
              <span className="seller_name"><AccountCircle className="icon"/>{renderName(lot.seller_id)}</span>
              {lot.status === 'OnSale' && <span className="countdown"><AccessTimeFilledIcon className="icon"/><Countdown date={getCountdownTime(lot)}/></span>}
            </div>
            {(isNotSeller && accountId && !lot.notSafe && lot.status === 'OnSale') && <div className="bid_price">
              <span className="buy-now_price">Buy now: <strong className="near-icon">{getBuyNowPrice(lot)}</strong></span>
              <button name="buy_now" onClick={(e) => bid(e, lot.lot_id, getBuyNowPrice(lot))}>Buy now</button>
              <input type="number" name="bid_input" className="large" onChange={(e) => onChangeBid(e)} ref={bidPrice}
                     placeholder={'min: ' + defaultValue} step="0.01" min={defaultValue} value={value}/>
              <button name="bid" onClick={(e) => bid(e, lot.lot_id, bidPrice.current.value)} disabled={bidButtonDisabled}>Bid</button>
              {bidPriceError && <span className="error-input">bid value should more than: {defaultValue}</span>}
            </div>}
            {(!isNotSeller && accountId && !lot.notSafe && lot.status === 'OnSale') && <Alert className="alert-container" severity="info">you cannot bid on your own lot</Alert>}
            {lot.notSafe && <Alert className="alert-container" severity="error">This lot has not passed the safety check</Alert>}
            <div className="bid_list">
              {lot.last_bidder_id ? <a className="button-link" onClick={(e) => openBidList(e)}>{showBidList ? 'Hide' : 'Show'} bid list</a> : ''}
              {showBidList ? (<ul className="bids_list">
                {bids.map((bid, i) =>
                  <Bids key={i} bid={bid}/>
                )}
              </ul>) : ''}
            </div>
          </div> :
          <Alert className="alert-container" severity="error">This lot is no more available</Alert>
        }
      </Box>
    </Modal>
  )
}

export default ModalBid;