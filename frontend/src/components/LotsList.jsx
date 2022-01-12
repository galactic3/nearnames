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
  const [selectedLot, setSelectedLot] = useState(false);
  const [selectedLotBids, setSelectedLotBids] = useState([]);

  const withdraw = async (lot, e) => {
    try {
      e.target.disabled = true;
      e.target.innerText = 'Loading...';
      await contract.lot_withdraw({'lot_id': lot.lot_id}, BOATLOAD_OF_GAS);
      history.push("/profile");
    } finally {
      e.target.disabled = false;
      e.target.innerText = 'Withdraw';
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

  const openBid = async (lot, bids) => {
    setModalBidShow(true);
    setSelectedLot(lot);
    setSelectedLotBids(bids);
  }

  const closeBid = async () => {
    setModalBidShow(false);
  }

  const setLotNotSafe = (lot) => {
    ls.set('NotSafeLots', notSafeLots + ', ' + lot.lot_id);
  }

  const bid = async (e, lot, value) => {
    e.target.disabled = true;
    const bid_price = toNear(value);
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
                 contract={contract} showStatus={props.showStatus} signedAccount={signedAccount}/>
          )}
          {props.lots.length === 0 ? <li>No lots available</li> : ''}
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
