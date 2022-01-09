import React, { useState } from 'react';
import Lot from "./Lot";
import ModalClaim from "./Claim";
import ModalBid from "./Bid";
import { BOATLOAD_OF_GAS, fetchBidSafety, toNear, LOCK_CONTRACT_HASHES } from "../utils";
import ModalAlert from "./Alert";
import ls from "local-storage";
import { useHistory } from "react-router-dom";

function LotsList(props) {
  const history = useHistory();
  const contract = props.contract;
  const config = props.nearConfig;
  const signedAccount = props.signedAccount;
  const notSafeLots = ls.get('NotSafeLots') || '';

  const [modalClaimShow, setModalClaimShow] = useState(false);
  const [modalBidShow, setModalBidShow] = useState(false);
  const [modalAlertShow, setModalAlertShow] = useState(false);
  const [alertContent, setAlertContent] = useState('');
  const [loaderShow, setLoaderShow] = useState(false);
  const [selectedLot, setSelectedLot] = useState(false);
  const [selectedLotBids, setSelectedLotBids] = useState([]);

  const withdraw = async (lot, e) => {
    try {
      e.target.disabled = true;
      setLoaderShow(true);
      await contract.lot_withdraw({'lot_id': lot.lot_id}, BOATLOAD_OF_GAS);
      history.push("/profile");
    } finally {
      setLoaderShow(false);
      e.target.disabled = false;
    }
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
    const { codeHash, accessKeysLen, lockerOwner } = await fetchBidSafety(lot.lot_id, props.near);

    const isSafe = LOCK_CONTRACT_HASHES.includes(codeHash) &&
      accessKeysLen === 0 &&
      lockerOwner === props.nearConfig.contractName;
    console.log(codeHash, accessKeysLen, lockerOwner);
    if (!isSafe) {
      alertOpen('account is not safe');
      setLotNotSafe(lot);
      lot.notSafe = true;
      e.target.disabled = false;
      return;
    }
    console.log(lot.lot_id, bid_price);
    contract.lot_bid({
      args: { lot_id: lot.lot_id },
      gas: BOATLOAD_OF_GAS,
      amount: bid_price,
      callbackUrl: new URL('/#/profile', window.location.origin),
    }).then(() => {
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
                 contract={contract} loader={loaderShow} showStatus={props.showStatus} signedAccount={signedAccount}/>
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
        signedAccount={signedAccount}
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
