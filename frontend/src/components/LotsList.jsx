import React, { useState } from 'react';
import Lot from "./Lot";
import ModalClaim from "./Claim";
import ModalBid from "./Bid";
import { BOATLOAD_OF_GAS } from "../utils";

function LotsList(props) {

  const contract = props.app.contract;
  const config = props.app.config;

  const [modalClaimShow, setModalClaimShow] = useState(false);
  const [modalBidShow, setModalBidShow] = useState(false);
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

  const claimOpen = async (lot) => {
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

  const closeBid = () => {
    setModalBidShow(false);
  }


  return (
    <div className="lots-container">
      { props.lots.length && props.name ? <h5 className="lots-title">Lots {props.name}</h5> : ''}
      { props.loader ?
        <div className='d-flex m-5 justify-content-center' key='1'>
          <div className='spinner-grow' role='status'>
            <span className='visually-hidden'>Loading...</span>
          </div>
        </div> :
        <ul className="lot_list">
          {props.lots.map((lot, i) =>
            <Lot lot={lot} key={i} bid={openBid} withdraw={withdraw} claim={claimOpen} contract={contract} loader={loaderShow} currentUser={props.app.accountId}/>
          )}
        </ul>
      }
      <ModalClaim
        show={modalClaimShow}
        lot={selectedLot}
        config={config}
        contract={contract}
        onHide={() => claimHide()}
      />
      <ModalBid
        open={modalBidShow}
        lot={selectedLot}
        bids={selectedLotBids}
        config={config}
        contract={contract}
        onClose={() => closeBid()}
      />
    </div>
  );
}

export default LotsList