import React, { useState } from 'react';
import Lot from "./Lot";
import ModalClaim from "./Claim";
import { fetchBidSafety, toNear, BOATLOAD_OF_GAS } from "../utils";



function LotsList(props) {

  const contract = props.app.contract;
  const config = props.app.config;

  const [modalShow, setModalShow] = useState(false);
  const [loaderShow, setLoaderShow] = useState(false);
  const [claimLotId, setClaimLotId] = useState(false);

  const withdraw = async (lot, e) => {
    e.target.disabled = true;
    setLoaderShow(true);
    await contract.lot_withdraw({'lot_id': lot.lot_id}, BOATLOAD_OF_GAS);
    setLoaderShow(false);
    e.target.disabled = false;
    props.getLots();
  };

  const claim = async (lot) => {
    setModalShow(true);
    setClaimLotId(lot.lot_id);
  };

  const claimHide = async () => {
    setModalShow(false);
    props.getLots();
  };

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
    contract.lot_bid({'lot_id': lot.lot_id}, BOATLOAD_OF_GAS, bid_price).then(() => {
        props.getLots();
    });
  };

  return (
    <div>
      { props.lots.length ? <h5>Lots {props.name}</h5> : ''}
      { props.loader ?
        <div className='d-flex m-5 justify-content-center' key='1'>
          <div className='spinner-grow' role='status'>
            <span className='visually-hidden'>Loading...</span>
          </div>
        </div> :
        <ul className="lot_list">
          {props.lots.map((lot, i) =>
            <Lot lot={lot} key={i} bid={bid} withdraw={withdraw} claim={claim} contract={contract} loader={loaderShow} currentUser={props.app.accountId}/>
          )}
        </ul>
      }
      <ModalClaim
        show={modalShow}
        lot={claimLotId}
        config={config}
        contract={contract}
        onHide={() => claimHide()}
      />
    </div>
  );
}

export default LotsList