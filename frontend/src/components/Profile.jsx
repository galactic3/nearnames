import React, { useEffect, useState } from 'react';
import Loader from './Loader';
import { BOATLOAD_OF_GAS, nearTo, renderName, loadListPaginated } from "../utils";
import LotsList from "./LotsList";

function Profile (props) {
  const profileId = props.signedAccount;
  const [profile, setProfile] = useState([]);
  const [lotsOffering, setLotsOffering] = useState([]);
  const [lotsBidding, setLotsBidding] = useState([]);
  const [lotsWon, setLotsWon] = useState([]);
  const [loader, setLoader] = useState(false);
  const [claimLoader, setClaimLoader] = useState(false);

  const contract = props.contract;

  const getLotsOffering = async () => {
    await loadListPaginated(
      args => contract.lot_list_offering_by({ profile_id: profileId, ...args }),
    ).then(setLotsOffering);
  }

  const getLotsBidding = async () => {
    await loadListPaginated(
      args => contract.lot_list_bidding_by({ profile_id: profileId, ...args }),
    ).then((lots) => {
      setLotsWon([]);
      setLotsBidding([]);
      lots.forEach((lot) => {
        if (lot.status === 'SaleSuccess' && profileId === lot.last_bidder_id) {
          setLotsWon(lotsWon => [...lotsWon, lot]);
        } else {
          setLotsBidding(lotsBidding => [...lotsBidding, lot]);
        }
      })
    });
  }

  const putLotOffering = (lot) => {
    const updatedLots = lotsOffering.map((l) => {
      if (lot && l.lot_id === lot.lot_id) {
        return lot;
      }
      return l;
    })
    setLotsOffering(updatedLots);
  }

  const putLotBidding = (lot) => {
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
    setLoader(true);
    await contract.profile_get({profile_id: profileId}).then(setProfile);
    await getLotsOffering();
    await getLotsBidding();
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
      <div className="mt-3">
        <div className="profile-container">
          <h5 className="profile-name"><strong>{renderName(profileId)}</strong></h5>
          <div className="profile-block"><strong>Available:</strong> <span className="rewards near-icon">{nearTo(profile.rewards_available)}</span></div>
          <div className="profile-block"><strong>Claimed:</strong> <span className="rewards near-icon">{nearTo(profile.rewards_claimed)}</span></div>
          <button className="claim-rewards" disabled={!parseFloat(profile.rewards_available) || claimLoader} onClick={(e) => claim(e)}>{claimLoader ? 'Claiming...' : 'Claim rewards'}</button>
        </div>
        <LotsList lots={lotsOffering} getLots={getLotsOffering} putLot={putLotOffering} showStatus={true} name={' you are selling'} {...props}/>
        <LotsList lots={lotsBidding} getLots={getLotsBidding} putLot={putLotBidding} showStatus={true} name={' you are bidding on'} {...props}/>
        <LotsList lots={lotsWon} getLots={getLotsBidding} putLot={putLotBidding} showStatus={true} name={' you won'} {...props}/>
      </div> :
      <div>Profile not found</div>
    }
    </div>
  );


}

export default Profile
