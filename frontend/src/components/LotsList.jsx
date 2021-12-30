import React, { useState } from 'react';
import Lot from "./Lot";
import ModalClaim from "./Claim";
import ModalBid from "./Bid";
import {BOATLOAD_OF_GAS, fetchBidSafety, toNear} from "../utils";
import ModalAlert from "./Alert";
import ls from "local-storage";

function LotsList(props) {

  const contract = props.app.contract;
  const config = props.app.config;
  const notSafeLots = ls.get('NotSafeLots') || '';

  const [modalClaimShow, setModalClaimShow] = useState(false);
  const [modalBidShow, setModalBidShow] = useState(false);
  const [modalAlertShow, setModalAlertShow] = useState(false);
  const [alertContent, setAlertContent] = useState('');
  const [loaderShow, setLoaderShow] = useState(false);
  const [selectedLot, setSelectedLot] = useState(false);
  const [selectedLotBids, setSelectedLotBids] = useState([]);

  const withdraw = async (lot, e) => {
    e.target.disabled = true;
    setLoaderShow(true);
    await contract.lot_withdraw({'lot_id': lot.lot_id}, BOATLOAD_OF_GAS);
    setLoaderShow(false);
    e.target.disabled = false;
    props.getLots();
  };

  const alertOpen = (text) => {
    setModalAlertShow(true);
    setAlertContent(text);
  };

  const alertHide = () => {
    setModalAlertShow(false);
  };


  const claimOpen = (lot) => {
    setModalClaimShow(true);
    setSelectedLot(lot);
  };

  const claimHide = async () => {
    setModalClaimShow(false);
    props.getLots();
  };

  const openBid = (lot, bids) => {
    setModalBidShow(true);
    setSelectedLot(lot);
    setSelectedLotBids(bids);
  }

  const closeBid = async () => {
    setModalBidShow(false);
    // props.getLots();
  }

  const setLotNotSafe = (lot) => {
    ls.set('NotSafeLots', notSafeLots + ', ' + lot.lot_id);
  }

  const bid = async (e, lot, value) => {
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
      alertOpen('account is not safe');
      setLotNotSafe(lot);
      lot.notSafe = true;
      e.target.disabled = false;
      return;
    }
    console.log(lot.lot_id, bid_price);
    contract.lot_bid({'lot_id': lot.lot_id}, BOATLOAD_OF_GAS, bid_price).then(() => {
      props.getLots();
    });
  };


  return (
    <div className="lots-container">
      { props.name ? <h5 className="lots-title">Lots {props.name}</h5> : ''}
      { props.loader ?
        <div className='d-flex m-5 justify-content-center' key='1'>
          <div className='spinner-grow' role='status'>
            <span className='visually-hidden'>Loading...</span>
          </div>
        </div> :
        <ul className="lot_list">
          {props.lots.map((lot, i) =>
            <Lot lot={lot} key={i} openBid={openBid} withdraw={withdraw} claim={claimOpen}
                 contract={contract} loader={loaderShow} showStatus={props.showStatus} currentUser={props.app.accountId}/>
          )}
        </ul>
      }
      <ModalClaim
        open={modalClaimShow}
        lot={selectedLot}
        config={config}
        contract={contract}
        onClose={() => claimHide()}
      />
      <ModalBid
        open={modalBidShow}
        lot={selectedLot}
        bid={bid}
        bids={selectedLotBids}
        currentUser={props.app.accountId}
        onClose={() => closeBid()}
      />
      <ModalAlert
        open={modalAlertShow}
        content={alertContent}
        onClose={() => alertHide()}
      />
    </div>
  );
}

export default LotsList