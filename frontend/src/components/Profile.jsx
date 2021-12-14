import React, { useEffect, useState } from 'react';
import { useParams } from 'react-router';
import Loader from './Loader';
import {BOATLOAD_OF_GAS, nearTo} from "../utils";
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
    <div>
    { loader ? <Loader/> : profile ?
      <div className="mt-3">
        <h5><strong>{profileId}</strong></h5>
        <p><strong>available:</strong> <span>{nearTo(profile.rewards_available)}</span><span title="NEAR Tokens">Ⓝ</span> <strong>claimed:</strong> <span>{nearTo(profile.rewards_claimed)}</span><span title="NEAR Tokens">Ⓝ</span></p>
        {claimLoader ? <Spinner className="mb-5" animation="grow" /> : <button name="claim_rewards" className="mb-5" disabled={!parseFloat(profile.rewards_available)} onClick={(e) => claim()}>Claim rewards</button> }
        <LotsList lots={lotsOffering} getLots={getLotsOffering} name={' offer'} {...props}/>
        <LotsList lots={lotsBidding} {...props} name={' bidding'}/>
      </div> :
      <div>Profile not found</div>
    }
    </div>
  );


}

export default Profile
