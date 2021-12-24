import React, { useEffect, useState } from 'react';
import { useParams } from 'react-router';
import Loader from './Loader';
import {BOATLOAD_OF_GAS, nearTo, renderName} from "../utils";
import {Spinner} from "react-bootstrap";
import LotsList from "./LotsList";

function Profile (props) {
  const { profileId } = useParams();
  const [profile, setProfile] = useState([]);
  const [lotsOffering, setLotsOffering] = useState([]);
  const [lotsBidding, setLotsBidding] = useState([]);
  const [loader, setLoader] = useState(false);
  const [claimLoader, setClaimLoader] = useState(false);

  const contract = props.app.contract;

  const getLotsOffering = async () => {
    await contract.lot_list_offering_by({profile_id: profileId}).then(setLotsOffering);
  }

  const getLotsBidding = async () => {
    await contract.lot_list_bidding_by({profile_id: profileId}).then(setLotsBidding);
  }

  useEffect(async () => {
    setLoader(true);
    await contract.profile_get({profile_id: profileId}).then(setProfile);
    setLoader(false);
    await getLotsOffering();
    await getLotsBidding();
  }, []);

  const claim = async () => {
    setClaimLoader(true);
    try {
      await contract.profile_rewards_claim({}, BOATLOAD_OF_GAS).then(() => {
        contract.profile_get({profile_id: profileId}).then((profile) => {
          setProfile(profile);
          console.log(profile);
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
        <LotsList lots={lotsOffering} getLots={getLotsOffering} showStatus={true} name={' offer'} {...props}/>
        <LotsList lots={lotsBidding} showStatus={true} name={' bidding'} {...props}/>
      </div> :
      <div>Profile not found</div>
    }
    </div>
  );


}

export default Profile
