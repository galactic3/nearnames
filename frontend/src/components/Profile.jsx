import React, { useEffect, useState } from 'react';
import Loader from './Loader';
import { BOATLOAD_OF_GAS, nearTo, renderName, loadListPaginated } from "../utils";
import LotsList from "./LotsList";

function Profile (props) {
  const profileId = props.currentUser.accountId;
  const [profile, setProfile] = useState([]);
  const [lotsOffering, setLotsOffering] = useState([]);
  const [lotsBidding, setLotsBidding] = useState([]);
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
    ).then(setLotsBidding);
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
          <div className="profile-block"><button name="claim_rewards" className="mb-5" disabled={!parseFloat(profile.rewards_available) || claimLoader} onClick={(e) => claim(e)}>{claimLoader ? 'Claiming...' : 'Claim rewards'}</button></div>
        </div>
        <LotsList lots={lotsOffering} getLots={getLotsOffering} showStatus={true} name={' you are selling'} {...props}/>
        <LotsList lots={lotsBidding} getLots={getLotsBidding} showStatus={true} name={' you are bidding on'} {...props}/>
      </div> :
      <div>Profile not found</div>
    }
    </div>
  );


}

export default Profile
