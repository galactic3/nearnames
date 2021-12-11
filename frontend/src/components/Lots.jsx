import React, { useState, useEffect } from 'react';
import {fetchBidSafety, toNear, BOATLOAD_OF_GAS} from "../utils";
import Lot from "./Lot";



function Lots(props) {

  const contract = props.app.contract;

  const [lots, setLots] = useState([]);

  useEffect(() => {
    contract.lot_list().then(setLots);
  }, []);

  const withdraw = async (lot) => {
    contract.lot_withdraw({'lot_id': lot.lot_id}, BOATLOAD_OF_GAS).then(() => {
        return contract.lot_list().then(setLots);
      }
    );
  };

  const claim = async (lot) => {
    contract.lot_claim({'lot_id': lot.lot_id, 'public_key': ''}, BOATLOAD_OF_GAS).then(() => {
        return contract.lot_list().then(setLots);
      }
    );
  };

  const bid = async (lot, e, value) => {
    const isBuyNowButton = e.target.name === 'buy_now';
    const bid_price = isBuyNowButton ? lot.buy_now_price : toNear(value);
    const { codeHash, accessKeysLen, lockerOwner } = await fetchBidSafety(lot.lot_id, props.app.near);
    const isSafe = codeHash === 'DKUq738xnns9pKjpv9GifM68UoFSmfnBYNp3hsfkkUFa' &&
                   accessKeysLen === 0 &&
                   lockerOwner === props.app.config.contractName;
    console.log(codeHash, accessKeysLen, lockerOwner);
    if (!isSafe) {
      alert("account is not safe");
    }
    contract.lot_bid({'lot_id': lot.lot_id}, BOATLOAD_OF_GAS, bid_price).then(() => {
        return contract.lot_list().then(setLots);
      }
    );
  };

  return (
    <div>
      {console.log(lots)}
      <h2>Lots</h2>
      { !lots.length ?
        <div className='d-flex m-5 justify-content-center' key='1'>
          <div className='spinner-grow' role='status'>
            <span className='visually-hidden'>Loading...</span>
          </div>
        </div> :
        <ul className="lot_list">
          {lots.map((lot, i) =>
            <Lot lot={lot} key={i} bid={bid} withdraw={withdraw} claim={claim} currentUser={props.app.accountId}/>
          )}
        </ul>
      }
    </div>
  );
}

export default Lots;
