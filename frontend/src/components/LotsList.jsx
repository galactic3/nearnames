import React, {useEffect, useState} from 'react';
import Lot from "./Lot";
import ModalClaim from "./Claim";
import ModalBid from "./Bid";
import {
  BOATLOAD_OF_GAS,
  toNear,
  getBuyNowPrice,
  getNextBidAmount,
} from "../utils";
import ModalAlert from "./Alert";
import { useHistory } from "react-router-dom";
import useConfirm from "../Hooks/useConfirm";
import Loader from "./Loader";
import ModalOffer from "./ReOffer";

function LotsList(props) {
  const history = useHistory();
  const contract = props.contract;
  const near = props.near;
  const config = props.nearConfig;
  const signedAccount = props.signedAccount;
  const { isConfirmed } = useConfirm();

  const [modalClaimShow, setModalClaimShow] = useState(false);
  const [modalBidShow, setModalBidShow] = useState(false);
  const [modalOfferShow, setModalOfferShow] = useState(false);
  const [modalAlertShow, setModalAlertShow] = useState(false);
  const [alertContent, setAlertContent] = useState('');
  const [selectedLot, setSelectedLot] = useState('');

  const withdraw = async (lot, e) => {
    try {
      e.target.disabled = true;
      e.target.innerText = 'Loading...';
      await contract.lot_withdraw({'lot_id': lot.lot_id}, BOATLOAD_OF_GAS);
      history.push("/profile");
    } catch (e) {
      e.target.innerText = 'Withdraw';
      let errorMessage = e.message;
      if (e.message.includes('expected no bids')) {
        errorMessage = "You can't withdraw because the lot had already bids";
      }
      if (e.message.includes('already withdrawn')) {
        errorMessage = "The lot has already been withdrawn";
      }
      alertOpen(errorMessage);
      e.target.innerText = 'Withdraw';
      console.error(e);
    } finally {
      await getLot(lot.lot_id);
      e.target.disabled = false;
    }
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

  const claimHide = async (claimSuccess) => {
    setModalClaimShow(false);
    if (claimSuccess) {
      await getLot(selectedLot.lot_id);
    }
    setSelectedLot('');
  };

  const openBid = async (lot) => {
    setModalBidShow(true);
    const selLot = await getLot(lot.lot_id);
    setSelectedLot(selLot);
  }

  const closeBid = async () => {
    setModalBidShow(false);
    setSelectedLot('');
  }

  const openOffer = async (lot) => {
    setModalOfferShow(true);
    setSelectedLot(lot);
  }

  const closeOffer = async (offerSuccess) => {
    setModalOfferShow(false);
    if (offerSuccess) {
      await getLot(selectedLot.lot_id);
    }
    setSelectedLot('');
  }

  const getLot = async (lotId) => {
    const lot = await contract.lot_get({lot_id: lotId});
    await updateLots(lot);
    return lot;
  }

  const updateLots = async (lot) => {
    if (lot) {
      await props.putLot(lot);
      setSelectedLot(lot);
    } else {
      await props.getLots();
    }
  }

  const bid = async (e, lotId, value) => {
    e.target.disabled = true;
    const lot = await getLot(lotId);
    if (lot.status !== 'OnSale') {
      alertOpen('Sorry lot no longer on sale');
      e.target.disabled = false;
      return;
    }
    const bid_price = toNear(value);
    if (bid_price && toNear(getNextBidAmount(lot)).cmp(bid_price) > 0) {
      alertOpen('Sorry lot next bid has changed');
      e.target.disabled = false;
      return;
    }
    if (bid_price.cmp(toNear(getBuyNowPrice(lot))) > 0) {
      const isConfirm = await isConfirmed(
        'Your bid price ' + value + ' NEAR is higher than the buy now price ' + getBuyNowPrice(lot) + ' NEAR. ' +
        'Are you sure you want to bid?'
      );
      if(!isConfirm) {
        e.target.disabled = false;
        return;
      }
    }

    await contract.lot_bid({
      args: { lot_id: lot.lot_id },
      gas: BOATLOAD_OF_GAS,
      amount: bid_price.toFixed(),
      callbackUrl: new URL('/#/profile', window.location.origin),
    });
  };

  useEffect( () => {
    props.getLots();
  }, []);

  return (
    <div className="lots-container">
      { props.name ? <h5 className="lots-title">Lots {props.name}</h5> : ''}
      { props.loader ?
        <Loader/> :
        <ul className="lot_list">
          {props.lots.map((lot, i) =>
            <Lot lot={lot} key={i} contract={contract} showStatus={props.showStatus} signedAccount={signedAccount}
                 openBid={openBid} signIn={props.signIn} withdraw={withdraw} claim={claimOpen} offer={openOffer}/>
          )}
          {props.lots.length === 0 ? <li className='lot_item'><div className="lot_info">No lots available</div></li> : ''}
        </ul>
      }
      <ModalClaim
        open={modalClaimShow}
        lot={selectedLot}
        config={config}
        contract={contract}
        onClose={(claimSuccess) => claimHide(claimSuccess)}
      />
      <ModalBid
        open={modalBidShow}
        lot={selectedLot}
        bid={bid}
        contract={contract}
        signedAccount={signedAccount}
        onClose={() => closeBid()}
      />
      <ModalOffer
        lot={selectedLot}
        open={modalOfferShow}
        near={near}
        contract={contract}
        onClose={(offerSuccess) => closeOffer(offerSuccess)}
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
