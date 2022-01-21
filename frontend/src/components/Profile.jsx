import React, {useEffect, useState} from 'react';
import Loader from './Loader';
import {BOATLOAD_OF_GAS, nearToFloor, renderName, loadListPaginated, fetchBidSafety} from "../utils";
import LotsList from "./LotsList";

function Profile (props) {
  const profileId = props.signedAccount;
  const [profile, setProfile] = useState([]);
  const [lotsOffering, setLotsOffering] = useState([]);
  const [lotsBidding, setLotsBidding] = useState([]);
  const [lotsWon, setLotsWon] = useState([]);
  const [loader, setLoader] = useState(true);
  const [lotsOfferLoader, setLotsOfferLoader] = useState(false);
  const [lotsBidLoader, setLotsBidLoader] = useState(false);
  const [lotsWonLoader, setLotsWonLoader] = useState(false);
  const [claimLoader, setClaimLoader] = useState(false);

  const contract = props.contract;

  const lotsChecked = async (lotsMemo) => {

    if (!lotsMemo.length) {
      return [];
    }

    await Promise.all(lotsMemo.map(async (l) => {
      const isSafe = await fetchBidSafety(l.lot_id, props.near, props.nearConfig);
      l.notSafe = !isSafe;
    }));

    return lotsMemo;

  }

  const getLotsOffering = async () => {

    let lotsMemo = [];

    console.time('lots offer fetch');

    setLotsOfferLoader(true);

    await loadListPaginated(
      args => contract.lot_list_offering_by({ profile_id: profileId, ...args }),
    ).then(async (lots) => {
      lotsMemo = lots;
      setLotsOffering(lotsMemo);
    });

    setLotsOfferLoader(false);

    console.timeEnd('lots offer fetch');

    console.time('lots offer check');

    const checked = await lotsChecked(lotsMemo);

    setLotsOffering([...checked]);

    console.timeEnd('lots offer check');

  }

  const getLotsBidding = async () => {

    let lotsMemo = [];

    setLotsBidLoader(true);

    console.time('lots bid fetch');

    await loadListPaginated(
      args => contract.lot_list_bidding_by({ profile_id: profileId, ...args }),
    ).then(async (lots) => {
      lotsMemo = lots.filter((lot) => {
        return lot.status === 'OnSale';
      });
      setLotsBidding(lotsMemo);
    });

    setLotsBidLoader(false);

    console.timeEnd('lots bid fetch');

    console.time('lots bid check');

    const checked = await lotsChecked(lotsMemo);

    setLotsBidding([...checked]);

    console.timeEnd('lots bid check');

  }

  const getLotsWon = async () => {

    let lotsMemo = [];

    console.time('lots won fetch');

    setLotsWonLoader(true);

    await loadListPaginated(
      args => contract.lot_list_bidding_by({ profile_id: profileId, ...args }),
    ).then(async (lots) => {
      lotsMemo = lots.filter((lot) => {
        return lot.status === 'SaleSuccess' && profileId === lot.last_bidder_id;
      });
      setLotsWon(lotsMemo);
    });

    setLotsWonLoader(false);

    console.timeEnd('lots won fetch');

    console.time('lots won check');

    const checked = await lotsChecked(lotsMemo);

    setLotsWon([...checked]);

    console.timeEnd('lots won check');

  }

  const putLotOffering = async (lot) => {
    const isSafe = await fetchBidSafety(lot.lot_id, props.near, props.nearConfig);
    lot.notSafe = !isSafe;
    const updatedLots = lotsOffering.map((l) => {
      if (lot && l.lot_id === lot.lot_id) {
        return lot;
      }
      return l;
    })
    setLotsOffering(updatedLots);
  }

  const putLotBidding = async (lot) => {
    const isSafe = await fetchBidSafety(lot.lot_id, props.near, props.nearConfig);
    lot.notSafe = !isSafe;
    const updatedLots = [...lotsWon, ...lotsBidding].map((l) => {
      if (l.lot_id === lot.lot_id) {
        return lot;
      }
      return l;
    });
    setLotsWon([]);
    setLotsBidding([]);
    updatedLots.forEach((lot) => {
      if (lot.status === 'SaleSuccess' && profileId === lot.last_bidder_id) {
        setLotsWon(lotsWon => [...lotsWon, lot]);
      } else {
        setLotsBidding(lotsBidding => [...lotsBidding, lot]);
      }
    })
  }

  useEffect(async () => {
    await contract.profile_get({profile_id: profileId}).then(setProfile);
    setLoader(false);
  }, []);

  const claim = async () => {
    setClaimLoader(true);
    try {
      await contract.profile_rewards_claim({}, BOATLOAD_OF_GAS).then(() => {
        contract.profile_get({profile_id: profileId}).then(async (profile) => {
          setProfile(profile);
          await props.updateBalance();
          setClaimLoader(false);
        });
      });
    } catch (e) {
      console.error(e);
      setClaimLoader(false);
    }
  };


  return (
    <div className="container">
    { loader ? <Loader/> : profile ?
      <div>
        <div className="profile-container">
          <h5 className="profile-name"><strong>{renderName(profileId)}</strong></h5>
          <div className="profile-block"><strong>Available:</strong> <span className="rewards near-icon">{nearToFloor(profile.rewards_available)}</span></div>
          <div className="profile-block"><strong>Claimed:</strong> <span className="rewards near-icon">{nearToFloor(profile.rewards_claimed)}</span></div>
          <button className="claim-rewards" disabled={!parseFloat(profile.rewards_available) || claimLoader} onClick={(e) => claim(e)}>{claimLoader ? 'Claiming...' : 'Claim rewards'}</button>
        </div>
        <LotsList lots={lotsOffering} getLots={getLotsOffering} putLot={putLotOffering} showStatus={true} loader={lotsOfferLoader} name={' you are selling'} {...props}/>
        <LotsList lots={lotsBidding} getLots={getLotsBidding} putLot={putLotBidding} showStatus={true} loader={lotsBidLoader} name={' you are bidding on'} {...props}/>
        <LotsList lots={lotsWon} getLots={getLotsWon} putLot={putLotBidding} showStatus={true} loader={lotsWonLoader} name={' you won'} {...props}/>
      </div> :
      <div className="profile-container">
        <h5 className="profile-name"><strong>Profile not found</strong></h5>
      </div>
    }
    </div>
  );


}

export default Profile
